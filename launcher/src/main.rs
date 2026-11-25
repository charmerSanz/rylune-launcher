#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod game;
mod auth;
mod ui;
use serde_json::{Value,json};
use std::{path::PathBuf,fs,sync::{Mutex},collections::HashMap};
use std::sync::{Arc,mpsc};
type R<T> = Result<T,String>;
const RYLUNE:&str="1.21.11";
struct Runtime { tokens:Mutex<HashMap<String,String>>, devices:Mutex<HashMap<String,auth::Device>> }
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
fn load_config()->R<Value>{let cfg=read_config()?;for p in cfg["profiles"].as_array().into_iter().flatten(){prepare_profile(p)?;}Ok(cfg)}
fn save_config(config:Value)->R<()>{
    let profiles=config["profiles"].as_array().ok_or("Missing profiles")?;
    if profiles.is_empty(){return Err("At least one profile is required".into())}
    let mut seen=std::collections::HashSet::new();
    for profile in profiles {let id=profile["id"].as_str().ok_or("Missing ID")?;if !seen.insert(id){return Err("Duplicate profile ID".into())}prepare_profile(profile)?;}
    let p=root()?.join("settings.json");let temp=p.with_extension("tmp");
    fs::write(&temp,serde_json::to_vec_pretty(&config).map_err(err)?).map_err(err)?;fs::rename(temp,p).map_err(err)
}
fn installed_versions()->R<Vec<String>>{
    let cfg=read_config()?;let mut result=Vec::new();
    for p in cfg["profiles"].as_array().into_iter().flatten(){let dir=profile_dir(p)?;let version=p["version"].as_str().unwrap_or("");if dir.join(".rylune-installed.json").is_file() && dir.join("versions").join(version).join(format!("{version}.jar")).is_file(){result.push(version.to_owned())}}
    result.sort();result.dedup();Ok(result)
}
fn open_folder(profile:Option<String>)->R<()>{
    let path=if let Some(id)=profile{let cfg=read_config()?;let p=cfg["profiles"].as_array().into_iter().flatten().find(|p|p["id"].as_str()==Some(id.as_str())).ok_or("Unknown profile")?;prepare_profile(p)?}else{root()?};
    open::that(path).map_err(err)
}
fn offline_account(name:String)->R<Value>{if !(3..=16).contains(&name.len())||!name.chars().all(|c|c.is_ascii_alphanumeric()||c=='_'){return Err("Use 3–16 letters, digits or underscores".into())}let mut b=md5::compute(format!("OfflinePlayer:{name}")).0;b[6]=(b[6]&15)|48;b[8]=(b[8]&63)|128;let h=b.iter().map(|x|format!("{x:02x}")).collect::<String>();Ok(json!({"id":h,"name":name,"type":"Offline"}))}
fn signed_in(state:&Runtime)->Vec<String>{state.tokens.lock().unwrap().keys().cloned().collect()}

#[derive(Clone)]
struct AppHandle {
    runtime: Arc<Runtime>,
    tx: mpsc::Sender<LauncherEvent>,
    ctx: eframe::egui::Context,
}
enum LauncherEvent {
    Log(String), Progress(u32), State(String), Finished(R<()>,bool),
    Catalog(R<Vec<String>>), Device(Value), Login(R<Value>),
}
impl AppHandle {
    fn send(&self,event:LauncherEvent){let _=self.tx.send(event);self.ctx.request_repaint();}
    fn emit<T:serde::Serialize>(&self,name:&str,value:T)->R<()>{
        let v=serde_json::to_value(value).map_err(err)?;
        match name {
            "launcher-log"=>self.send(LauncherEvent::Log(v.as_str().unwrap_or("").to_owned())),
            "launcher-progress"=>self.send(LauncherEvent::Progress(v.as_u64().unwrap_or(0).min(100) as u32)),
            "launcher-state"=>self.send(LauncherEvent::State(v.as_str().unwrap_or("").to_owned())),
            _=>{}
        }
        Ok(())
    }
}
fn fetch_catalog()->R<Vec<String>>{
    let data=game::get_json(&game::client()?,game::MANIFEST)?;
    let versions=data["versions"].as_array().ok_or("Invalid version catalog")?;
    Ok(versions.iter().filter(|v|v["type"]=="release").filter_map(|v|v["id"].as_str()).filter(|v|supported_version(v)).map(str::to_owned).collect())
}

fn main()->eframe::Result<()> {
    let icon=image::load_from_memory(include_bytes!("../public/logo.png")).ok().map(|i|{
        let i=i.resize_exact(64,64,image::imageops::FilterType::Lanczos3).into_rgba8();
        eframe::egui::IconData{width:i.width(),height:i.height(),rgba:i.into_raw()}
    });
    let mut viewport=eframe::egui::ViewportBuilder::default().with_inner_size([1440.0,900.0]).with_min_inner_size([980.0,680.0]);
    if let Some(icon)=icon {viewport=viewport.with_icon(icon);}
    let options=eframe::NativeOptions{viewport,vsync:true,renderer:eframe::Renderer::Glow,..Default::default()};
    eframe::run_native("Rylune Client",options,Box::new(|cc|Ok(Box::new(ui::Launcher::new(cc)))))
}
