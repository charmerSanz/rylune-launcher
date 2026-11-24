#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod game; mod auth;
use serde_json::{Value,json};
use std::{path::PathBuf,fs,sync::{Mutex,atomic::{AtomicBool,Ordering}},collections::HashMap};
use tauri::{AppHandle,Emitter,Manager,State};
type R<T> = Result<T,String>;
const RYLUNE:&str="1.21.11";
struct Runtime { busy:AtomicBool, tokens:Mutex<HashMap<String,String>>, devices:Mutex<HashMap<String,auth::Device>> }
fn root()->R<PathBuf>{let base=std::env::var_os("APPDATA").or_else(||std::env::var_os("HOME")).ok_or("No user data directory")?;let p=PathBuf::from(base).join("Rylune");fs::create_dir_all(&p).map_err(err)?;Ok(p)}
fn err(e:impl std::fmt::Display)->String{e.to_string()}
fn log(app:&AppHandle,s:impl AsRef<str>){let _=app.emit("launcher-log",s.as_ref());}
fn safe_id(s:&str)->bool{!s.is_empty()&&s.len()<=128&&s!="."&&s!=".."&&s.chars().all(|c|c.is_ascii_alphanumeric()||"._+-".contains(c))}
// Only stable modern releases in the requested range are supported.
fn supported_version(v:&str)->bool {
    let parts:Vec<&str>=v.split('.').collect();
    let nums:Option<Vec<u32>>=parts.iter().map(|x|x.parse::<u32>().ok()).collect();
    let Some(n)=nums else{return false};
    if n.iter().map(|x|x.to_string()).collect::<Vec<_>>().join(".")!=v{return false}
    n.as_slice()==[1,21,11] || (n.len()>=2 && n.len()<=3 && n[0]==26 && (1..=3).contains(&n[1]))
}
fn profile_dir(p:&Value)->R<PathBuf>{
    let id=p["id"].as_str().ok_or("Missing profile ID")?;
    let v=p["version"].as_str().ok_or("Missing game version")?;
    if !safe_id(id)||!supported_version(v){return Err("Invalid profile or unsupported Minecraft version".into())}
    let name=if id==v {v.to_owned()}else{format!("{v}-{id}")};
    Ok(root()?.join("profiles").join(name))
}
fn prepare_profile(p:&Value)->R<PathBuf>{
    let dir=profile_dir(p)?;
    for sub in ["assets","libraries","versions","natives","mods","config","saves","resourcepacks","shaderpacks","screenshots","logs","crash-reports","runtime"] {
        fs::create_dir_all(dir.join(sub)).map_err(err)?;
    }
    Ok(dir)
}
fn read_config()->R<Value>{
    let path=root()?.join("settings.json");
    if path.exists(){serde_json::from_slice(&fs::read(path).map_err(err)?).map_err(err)}else{
        Ok(json!({"profiles":[{"id":RYLUNE,"name":"Minecraft 1.21.11","version":RYLUNE,"ram":2048,"java":""}],"accounts":[],"profile":RYLUNE,"account":"","client_id":""}))
    }
}
#[tauri::command]fn load_config()->R<Value>{let cfg=read_config()?;for p in cfg["profiles"].as_array().into_iter().flatten(){prepare_profile(p)?;}Ok(cfg)}
#[tauri::command]fn save_config(config:Value)->R<()>{
    let profiles=config["profiles"].as_array().ok_or("Missing profiles")?;
    if profiles.is_empty(){return Err("At least one profile is required".into())}
    let mut seen=std::collections::HashSet::new();
    for profile in profiles {let id=profile["id"].as_str().ok_or("Missing ID")?;if !seen.insert(id){return Err("Duplicate profile ID".into())}prepare_profile(profile)?;}
    let p=root()?.join("settings.json");let temp=p.with_extension("tmp");
    fs::write(&temp,serde_json::to_vec_pretty(&config).map_err(err)?).map_err(err)?;fs::rename(temp,p).map_err(err)
}
#[tauri::command]fn installed_versions()->R<Vec<String>>{
    let cfg=read_config()?;let mut result=Vec::new();
    for p in cfg["profiles"].as_array().into_iter().flatten(){let dir=profile_dir(p)?;let version=p["version"].as_str().unwrap_or("");if dir.join(".rylune-installed.json").is_file() && dir.join("versions").join(version).join(format!("{version}.jar")).is_file(){result.push(version.to_owned())}}
    result.sort();result.dedup();Ok(result)
}
#[tauri::command]fn open_folder(profile:Option<String>)->R<()>{
    let path=if let Some(id)=profile{let cfg=read_config()?;let p=cfg["profiles"].as_array().into_iter().flatten().find(|p|p["id"].as_str()==Some(id.as_str())).ok_or("Unknown profile")?;prepare_profile(p)?}else{root()?};
    open::that(path).map_err(err)
}
#[tauri::command]fn offline_account(name:String)->R<Value>{if !(3..=16).contains(&name.len())||!name.chars().all(|c|c.is_ascii_alphanumeric()||c=='_'){return Err("Use 3–16 letters, digits or underscores".into())}let mut b=md5::compute(format!("OfflinePlayer:{name}")).0;b[6]=(b[6]&15)|48;b[8]=(b[8]&63)|128;let h=b.iter().map(|x|format!("{x:02x}")).collect::<String>();Ok(json!({"id":h,"name":name,"type":"Offline"}))}
#[tauri::command]fn signed_in(state:State<Runtime>)->Vec<String>{state.tokens.lock().unwrap().keys().cloned().collect()}
#[tauri::command]async fn catalog()->R<Value>{tauri::async_runtime::spawn_blocking(|| -> R<Value> {let mut m=game::get_json(&game::client()?,game::MANIFEST)?;let list=m["versions"].as_array_mut().ok_or("Invalid manifest")?;list.retain(|v|v["type"]=="release"&&v["id"].as_str().is_some_and(supported_version));Ok(m)}).await.map_err(err)?}
#[tauri::command]async fn start_login(app:AppHandle,client_id:String)->R<Value>{tauri::async_runtime::spawn_blocking(move||auth::start(&app,&client_id)).await.map_err(err)?}
#[tauri::command]async fn poll_login(app:AppHandle,session:String)->R<Value>{tauri::async_runtime::spawn_blocking(move||auth::poll(&app,&session)).await.map_err(err)?}
#[tauri::command]fn sign_out(state:State<Runtime>,id:String){state.tokens.lock().unwrap().remove(&id);}
#[tauri::command]async fn run_game(app:AppHandle,profile:Value,account:Value,install_only:bool)->R<()>{tauri::async_runtime::spawn_blocking(move||{let state=app.state::<Runtime>();if state.busy.swap(true,Ordering::SeqCst){return Err("Another operation is active".into())}let _=app.emit("launcher-state","preparing");let result=game::run(&app,&profile,&account,install_only);state.busy.store(false,Ordering::SeqCst);let _=app.emit("launcher-state",if result.is_ok(){"idle"}else{"failed"});result}).await.map_err(err)?}
fn main(){tauri::Builder::default().manage(Runtime{busy:AtomicBool::new(false),tokens:Mutex::new(HashMap::new()),devices:Mutex::new(HashMap::new())}).invoke_handler(tauri::generate_handler![load_config,save_config,installed_versions,open_folder,offline_account,signed_in,catalog,start_login,poll_login,sign_out,run_game]).run(tauri::generate_context!()).expect("Rylune failed to start");}
