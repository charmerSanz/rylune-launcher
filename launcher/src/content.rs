//! Modrinth content and isolated .mrpack import.
use super::*;
use reqwest::blocking::Client;
use sha2::{Digest,Sha512};
use std::{collections::{HashMap,HashSet},io::{Read,Write},path::{Path,PathBuf},time::Duration};
const API:&str="https://api.modrinth.com/v2";
const LIMIT:u64=1024*1024*1024;
const DIRS:[&str;3]=["mods","resourcepacks","shaderpacks"];
pub const TYPES:[(&str,&str);4]=[("Mods","mod"),("Modpacks","modpack"),("Resource packs","resourcepack"),("Shaders","shader")];
fn client()->R<Client>{
    Client::builder().user_agent(concat!("charmerSanz/Rylune-Launcher/",env!("CARGO_PKG_VERSION")," (https://github.com/charmerSanz/rylune-launcher)"))
        .timeout(Duration::from_secs(120)).redirect(reqwest::redirect::Policy::custom(|attempt|{
            if attempt.previous().len()>=5||!trusted_host(attempt.url()){attempt.error("Untrusted or excessive redirect")}else{attempt.follow()}
        })).build().map_err(err)
}
fn strv<'a>(v:&'a Value,k:&str)->R<&'a str>{v[k].as_str().ok_or_else(||format!("Missing Modrinth field: {k}"))}
fn get(c:&Client,url:&str)->R<Value>{c.get(url).send().map_err(err)?.error_for_status().map_err(err)?.json().map_err(err)}
fn id(s:&str)->R<&str>{if safe_id(s){Ok(s)}else{Err("Invalid project or version ID".into())}}
fn array(v:&Value,k:&str,s:&str)->bool{v[k].as_array().is_some_and(|a|a.iter().any(|x|x.as_str()==Some(s)))}
fn folder(kind:&str)->R<&'static str>{match kind{"mod"=>Ok("mods"),"resourcepack"=>Ok("resourcepacks"),"shader"=>Ok("shaderpacks"),_=>Err("Unsupported content type".into())}}
fn is_pack(kind:&str)->bool{kind=="modpack"}
fn check_kind(kind:&str)->R<()>{if TYPES.iter().any(|(_,k)|*k==kind){Ok(())}else{Err("Invalid content type".into())}}
pub fn search(query:&str,kind:&str,version:&str,sort:&str,offset:u32)->R<Value>{
    check_kind(kind)?;
    let mut facets=vec![vec![format!("project_type:{kind}")],vec![format!("versions:{version}")]];
    if kind=="mod"||kind=="modpack"{facets.push(vec!["categories:fabric".into()]);}
    let facets=serde_json::to_string(&facets).map_err(err)?;
    let offset=offset.to_string();
    client()?.get(format!("{API}/search")).query(&[("query",query),("facets",facets.as_str()),("index",sort),("limit","20"),("offset",offset.as_str())])
        .send().map_err(err)?.error_for_status().map_err(err)?.json().map_err(err)
}
fn version(c:&Client,project:&str,kind:&str,mc:&str,pinned:Option<&str>)->R<Value>{
    let compatible=|v:&Value|array(v,"game_versions",mc)&&(!(kind=="mod"||kind=="modpack")||array(v,"loaders","fabric"));
    if let Some(pin)=pinned{
        let v=get(c,&format!("{API}/version/{}",id(pin)?))?;
        if v["project_id"].as_str()!=Some(project)||!compatible(&v){return Err(format!("Required dependency {project} has no compatible pinned version for {mc} Fabric"))}
        return Ok(v)
    }
    let games=json!([mc]).to_string();
    let mut request=c.get(format!("{API}/project/{}/version",id(project)?)).query(&[("game_versions",&games)]);
    if kind=="mod"||kind=="modpack"{request=request.query(&[("loaders","[\"fabric\"]")]);}
    let data:Value=request.send().map_err(err)?.error_for_status().map_err(err)?.json().map_err(err)?;
    let list=data.as_array().ok_or("Invalid Modrinth version list")?;
    list.iter().find(|v|compatible(v)&&v["version_type"]=="release").or_else(||list.iter().find(|v|compatible(v))).cloned().ok_or_else(||format!("No compatible {kind} version for Minecraft {mc}"))
}
fn primary(v:&Value)->R<&Value>{
    let files=v["files"].as_array().ok_or("Missing downloadable files")?;
    files.iter().find(|f|f["primary"]==true).or_else(||if files.len()==1{files.first()}else{None}).ok_or_else(|| "No unambiguous primary download".to_owned())
}
fn filename(name:&str)->R<&str>{
    let stem=name.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved=matches!(stem.as_str(),"CON"|"PRN"|"AUX"|"NUL"|"COM1"|"COM2"|"COM3"|"COM4"|"COM5"|"COM6"|"COM7"|"COM8"|"COM9"|"LPT1"|"LPT2"|"LPT3"|"LPT4"|"LPT5"|"LPT6"|"LPT7"|"LPT8"|"LPT9");
    if name.is_empty()||name.len()>240||name=="."||name==".."||name.ends_with('.')||name.ends_with(' ')||name.chars().any(|c|c.is_control()||"/\\:<>\"|?*".contains(c))||reserved{
        return Err(format!("Unsafe filename: {name}"))
    }
    Ok(name)
}
fn allowed_path(path:&str)->R<PathBuf>{
    if path.len()>1024{return Err("Pack path too long".into())}
    let mut out=PathBuf::new();for p in path.split('/'){out.push(filename(p)?);}
    let first=path.split('/').next().unwrap_or("").to_ascii_lowercase();
    if !["mods","resourcepacks","shaderpacks","config","defaultconfigs","scripts","kubejs","saves","datapacks","options.txt","servers.dat","resourcepack","journeymap","xaero","optionsof.txt","global_packs"].contains(&first.as_str()){
        return Err(format!("Unsupported or protected modpack path: {path}"))
    }
    Ok(out)
}
fn sha(p:&Path)->R<String>{
    let mut f=fs::File::open(p).map_err(err)?;let mut h=Sha512::new();let mut buf=[0u8;65536];
    loop{let n=f.read(&mut buf).map_err(err)?;if n==0{break}h.update(&buf[..n]);}
    Ok(format!("{:x}",h.finalize()))
}
fn trusted_host(u:&reqwest::Url)->bool{u.scheme()=="https"&&u.username().is_empty()&&u.password().is_none()&&matches!(u.host_str(),Some("cdn.modrinth.com"|"github.com"|"raw.githubusercontent.com"|"objects.githubusercontent.com"|"release-assets.githubusercontent.com"|"gitlab.com"|"api.modrinth.com"))}
fn download(c:&Client,url:&str,hash:&str,size:Option<u64>,out:&Path)->R<()>{
    if hash.len()!=128||!hash.chars().all(|c|c.is_ascii_hexdigit()){return Err("Missing or invalid SHA-512 checksum".into())}
    let u=reqwest::Url::parse(url).map_err(err)?;
    if u.scheme()!="https"||!u.username().is_empty()||u.password().is_some(){return Err("Only public HTTPS content URLs are supported".into())}
    // Limit source hosts to trusted distribution domains; validate redirected destination too.
    let allowed=trusted_host;
    if !allowed(&u){return Err(format!("Unsupported download host: {}",u.host_str().unwrap_or("?")))}
    if size.is_some_and(|n|n>LIMIT){return Err("File exceeds 1 GiB download limit".into())}
    fs::create_dir_all(out.parent().ok_or("Invalid download path")?).map_err(err)?;
    let mut last=String::new();
    for _ in 0..3{
        let result=(||->R<()>{
            let response=c.get(u.clone()).send().map_err(err)?.error_for_status().map_err(err)?;
            if !allowed(response.url()){return Err("Untrusted redirect destination".into())}
            let mut response=response.take(LIMIT+1);
            let mut file=fs::File::create(out).map_err(err)?;
            let n=std::io::copy(&mut response,&mut file).map_err(err)?;file.flush().map_err(err)?;drop(file);
            if n>LIMIT||size.is_some_and(|s|n!=s)||!sha(out)?.eq_ignore_ascii_case(hash){return Err("Content size or checksum mismatch".into())}
            Ok(())
        })();
        match result{Ok(())=>return Ok(()),Err(e)=>{last=e;let _=fs::remove_file(out);}}
    }
    Err(last)
}
fn download_file(c:&Client,f:&Value,out:&Path)->R<()>{
    download(c,strv(f,"url")?,strv(&f["hashes"],"sha512")?,f["size"].as_u64(),out)
}
fn registry(dir:&Path)->R<Vec<Value>>{
    let p=dir.join(".rylune-content.json");
    if !p.exists(){return Ok(vec![])}
    serde_json::from_slice(&fs::read(p).map_err(err)?).map_err(err)
}
fn save_registry(dir:&Path,list:&[Value])->R<()>{
    let p=dir.join(".rylune-content.json");let tmp=dir.join(".rylune-content.tmp");
    fs::write(&tmp,serde_json::to_vec_pretty(list).map_err(err)?).map_err(err)?;fs::rename(tmp,p).map_err(err)
}
fn content_path(dir:&Path,kind:&str,name:&str)->R<PathBuf>{
    let path=dir.join(folder(kind)?).join(filename(name)?);
    let meta=fs::symlink_metadata(path.parent().ok_or("Invalid path")?).map_err(err)?;
    if !meta.is_dir()||meta.file_type().is_symlink(){return Err("Content directory is not a regular directory".into())}
    Ok(path)
}
fn ext_ok(kind:&str,name:&str)->bool{
    let s=name.to_ascii_lowercase();match kind{"mod"=>s.ends_with(".jar"),"resourcepack"|"shader"=>s.ends_with(".zip"),"modpack"=>s.ends_with(".mrpack"),_=>false}
}
struct Staging(PathBuf);
impl Drop for Staging{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn stage(dir:&Path)->R<Staging>{
    let p=dir.join(format!(".rylune-stage-{}",uuid::Uuid::new_v4()));
    fs::create_dir_all(&p).map_err(err)?;Ok(Staging(p))
}
struct Resolver<'a>{c:&'a Client,mc:&'a str,plan:Vec<Value>,chosen:HashMap<String,String>,visiting:HashSet<String>}
impl Resolver<'_>{
    fn visit(&mut self,project:&str,pinned:Option<&str>,expected:Option<&str>,depth:u8)->R<()>{
        if depth>32||self.plan.len()+self.visiting.len()>128{return Err("Dependency graph exceeds supported size".into())}
        if let Some(v)=self.chosen.get(project){if pinned.is_some_and(|p|p!=v){return Err(format!("Conflicting required versions for {project}"))}return Ok(())}
        let info=get(self.c,&format!("{API}/project/{}",id(project)?))?;
        let kind=strv(&info,"project_type")?;
        if expected.is_some_and(|k|k!=kind){return Err("Unexpected project type".into())}
        folder(kind)?;
        if info["client_side"]=="unsupported"{return Err(format!("{} is server-only",info["title"].as_str().unwrap_or(project)))}
        let v=version(self.c,project,kind,self.mc,pinned)?;
        let vid=strv(&v,"id")?.to_owned();
        self.chosen.insert(project.into(),vid.clone());
        self.visiting.insert(project.into());
        for dep in v["dependencies"].as_array().into_iter().flatten(){
            if dep["dependency_type"]!="required"{continue}
            let pin=dep["version_id"].as_str();
            let dep_id=if let Some(p)=dep["project_id"].as_str(){p.to_owned()}else if let Some(pin)=pin{strv(&get(self.c,&format!("{API}/version/{}",id(pin)?))?,"project_id")?.to_owned()}else{return Err("Required dependency cannot be resolved automatically".into())};
            self.visit(&dep_id,pin,None,depth+1)?;
        }
        let file=primary(&v)?;
        let name=strv(file,"filename")?;
        filename(name)?;
        if !ext_ok(kind,name){return Err(format!("Unexpected primary file type: {name}"))}
        self.visiting.remove(project);
        self.plan.push(json!({"project":project,"version":vid,"kind":kind,"name":name,"title":info["title"],"file":file,"dependencies":v["dependencies"]}));
        Ok(())
    }
}
pub fn install(app:&AppHandle,p:&Value,project:&str,kind:&str,update:bool)->R<Option<Value>>{
    check_kind(kind)?;
    let mc=strv(p,"version")?;
    let c=client()?;
    if is_pack(kind){
        let v=version(&c,project,kind,mc,None)?;
        let tmp=stage(&root()?.join("profiles"))?;
        let path=tmp.0.join("pack.mrpack");download_file(&c,primary(&v)?,&path)?;
        return import_pack(app,&path).map(Some)
    }
    let dir=prepare_profile(p)?;
    let mut reg=registry(&dir)?;
    let mut resolver=Resolver{c:&c,mc,plan:vec![],chosen:HashMap::new(),visiting:HashSet::new()};
    resolver.visit(project,None,Some(kind),0)?;
    for item in &resolver.plan{
        for dep in item["dependencies"].as_array().into_iter().flatten(){
            if dep["dependency_type"]!="incompatible"{continue}
            let pid=dep["project_id"].as_str();let vid=dep["version_id"].as_str();
            if (pid.is_some()||vid.is_some())&&reg.iter().chain(resolver.plan.iter()).any(|x|pid.is_none_or(|p|x["project"].as_str()==Some(p))&&vid.is_none_or(|v|x["version"].as_str()==Some(v))){
                return Err(format!("{} declares a conflict with installed or selected content",item["title"].as_str().unwrap_or("Project")))
            }
        }
    }
    let stage=stage(&dir)?;
    let mut replacements:Vec<(PathBuf,PathBuf,Value,Option<PathBuf>)>=Vec::new();
    let mut seen=HashSet::new();
    for item in resolver.plan{
        let pid=strv(&item,"project")?;
        let k=strv(&item,"kind")?;
        let name=strv(&item,"name")?;
        let hash=strv(&item["file"]["hashes"],"sha512")?;
        let existing=reg.iter().find(|x|x["project"].as_str()==Some(pid)).cloned();
        let mut old=None;
        if let Some(e)=existing{
            let original=content_path(&dir,strv(&e,"kind")?,strv(&e,"name")?)?;
            let disabled=original.with_file_name(format!("{}.disabled",strv(&e,"name")?));
            if disabled.exists(){return Err(format!("Enable {} before installing or updating its dependents",e["title"].as_str().unwrap_or(pid)))}
            if e["version"]==item["version"]&&original.is_file()&&sha(&original)?.eq_ignore_ascii_case(hash){continue}
            if pid!=project||!update{return Err(format!("Existing version of {} conflicts; update or remove it in Library first",e["title"].as_str().unwrap_or(pid)))}
            // Never remove a locally modified managed file.
            if original.exists(){if fs::symlink_metadata(&original).is_ok_and(|m|m.file_type().is_symlink())||!original.is_file()||sha(&original)?!=strv(&e,"sha512")?{return Err("Existing managed file was modified; remove it manually before updating".into())}old=Some(original);}
        }
        let dest=content_path(&dir,k,name)?;
        let disabled=dest.with_file_name(format!("{name}.disabled"));
        if disabled.exists(){return Err(format!("Disabled file already exists: {name}"))}
        if !seen.insert(dest.to_string_lossy().to_ascii_lowercase()){return Err("Duplicate download filename in dependency plan".into())}
        if fs::symlink_metadata(&dest).is_ok_and(|m|m.file_type().is_symlink()){return Err("Refusing to replace a symbolic link".into())}
        if dest.exists()&&old.as_ref()!=Some(&dest){
            if dest.is_file()&&sha(&dest)?.eq_ignore_ascii_case(hash){reg.retain(|x|x["project"].as_str()!=Some(pid));reg.push(json!({"project":pid,"version":item["version"],"kind":k,"name":name,"title":item["title"],"sha512":hash}));continue}
            return Err(format!("File already exists: {name}. Remove or rename it in Library first."))
        }
        log(app,format!("Downloading {}",item["title"].as_str().unwrap_or(name)));
        let src=stage.0.join(format!("{}.download",replacements.len()));
        download_file(&c,&item["file"],&src)?;
        let record=json!({"project":pid,"version":item["version"],"kind":k,"name":name,"title":item["title"],"sha512":hash});
        replacements.push((src,dest,record,old));
    }
    let mut moved:Vec<(PathBuf,Option<(PathBuf,PathBuf)>)>=Vec::new();
    let result=(||->R<()>{
        for (i,(src,dest,record,old)) in replacements.iter().enumerate(){
            let backup=if let Some(old)=old{let backup=stage.0.join(format!("{i}.backup"));fs::rename(old,&backup).map_err(err)?;Some((old.clone(),backup))}else{None};
            if let Err(e)=fs::rename(src,dest){if let Some((old,backup))=&backup{let _=fs::rename(backup,old);}return Err(err(e))}
            moved.push((dest.clone(),backup));
            reg.retain(|x|x["project"]!=record["project"]);reg.push(record.clone());
        }
        save_registry(&dir,&reg)
    })();
    if let Err(e)=result{for (dest,backup) in moved.into_iter().rev(){let _=fs::remove_file(dest);if let Some((old,backup))=backup{let _=fs::rename(backup,old);}}return Err(e)}
    Ok(None)
}
pub fn library(p:&Value)->R<Vec<Value>>{
    let dir=prepare_profile(p)?;
    let reg=registry(&dir)?;
    let mut results=vec![];
    for (kind,sub) in [("mod",DIRS[0]),("resourcepack",DIRS[1]),("shader",DIRS[2])]{
        let subdir=dir.join(sub);
        if fs::symlink_metadata(&subdir).map_err(err)?.file_type().is_symlink(){continue}
        for f in fs::read_dir(subdir).map_err(err)?{
            let f=f.map_err(err)?;
            if !f.file_type().map_err(err)?.is_file(){continue}
            let name=f.file_name().to_string_lossy().into_owned();
            let bare=name.strip_suffix(".disabled").unwrap_or(&name);
            if !ext_ok(kind,bare){continue}
            let meta=reg.iter().find(|x|x["name"].as_str()==Some(bare)&&x["kind"]==kind);
            results.push(json!({"kind":kind,"name":name,"title":meta.and_then(|m|m["title"].as_str()).unwrap_or(bare),"project":meta.and_then(|m|m["project"].as_str()),"version":meta.and_then(|m|m["version"].as_str()),"enabled":name==bare}));
        }
    }
    results.sort_by_key(|v|v["title"].as_str().unwrap_or("").to_ascii_lowercase());Ok(results)
}
pub fn change(p:&Value,item:&Value,remove:bool)->R<()>{
    let dir=prepare_profile(p)?;let kind=strv(item,"kind")?;let name=strv(item,"name")?;
    let bare=name.strip_suffix(".disabled").unwrap_or(name);if !ext_ok(kind,bare){return Err("Invalid content file".into())}
    let path=content_path(&dir,kind,name)?;
    if !fs::symlink_metadata(&path).map_err(err)?.file_type().is_file(){return Err("Not a regular content file".into())}
    if remove{ // Keep a recoverable copy instead of permanently deleting.
        let trash=dir.join(".rylune-removed");fs::create_dir_all(&trash).map_err(err)?;
        fs::rename(&path,trash.join(format!("{}-{name}",uuid::Uuid::new_v4()))).map_err(err)?;
        let mut list=registry(&dir)?;list.retain(|v|!(v["kind"]==kind&&v["name"]==bare));save_registry(&dir,&list)?;
    }else{
        let new=if name==bare{format!("{name}.disabled")}else{bare.into()};
        let dest=content_path(&dir,kind,&new)?;
        if dest.exists(){return Err("Target filename already exists".into())}fs::rename(path,dest).map_err(err)?;
    }Ok(())
}
pub fn import_local(app:&AppHandle,p:&Value,path:&str,kind:&str)->R<Option<Value>>{
    let path=PathBuf::from(path.trim().trim_matches('"'));
    if kind=="modpack"{return import_pack(app,&path).map(Some)}
    let name=path.file_name().and_then(|s|s.to_str()).ok_or("Invalid file path")?;
    if !ext_ok(kind,name)||!path.is_file(){return Err("Select an existing file matching the content type".into())}
    let dir=prepare_profile(p)?;let dest=content_path(&dir,kind,name)?;
    if dest.exists()||dest.with_file_name(format!("{name}.disabled")).exists(){return Err("A file with this name already exists".into())}
    let stage=stage(&dir)?;let tmp=stage.0.join("import");fs::copy(&path,&tmp).map_err(err)?;fs::rename(tmp,dest).map_err(err)?;Ok(None)
}
pub fn import_pack(app:&AppHandle,path:&Path)->R<Value>{
    if !path.is_file()||path.extension().and_then(|s|s.to_str()).is_none_or(|s|!s.eq_ignore_ascii_case("mrpack")){return Err("Choose a valid .mrpack file".into())}
    let mut archive=zip::ZipArchive::new(fs::File::open(path).map_err(err)?).map_err(err)?;
    if archive.len()>20000{return Err("Modpack has too many archive entries".into())}
    let manifest:Value={let f=archive.by_name("modrinth.index.json").map_err(err)?;if f.size()>8*1024*1024{return Err("Manifest too large".into())}serde_json::from_reader(f).map_err(err)?};
    if manifest["formatVersion"]!=1||manifest["game"]!="minecraft"{return Err("Unsupported .mrpack format".into())}
    let mc=strv(&manifest["dependencies"],"minecraft")?;
    if !supported_version(mc){return Err(format!("Minecraft {mc} is outside Rylune's supported range"))}
    let loader=strv(&manifest["dependencies"],"fabric-loader").map_err(|_|"Only Fabric modpacks are currently supported".to_owned())?;
    if !safe_id(loader)||["forge","neoforge","quilt-loader"].iter().any(|k|manifest["dependencies"].get(*k).is_some()){return Err("Unsupported or mixed mod loader".into())}
    let c=client()?;
    let fabric=game::get_json(&c,&format!("https://meta.fabricmc.net/v2/versions/loader/{mc}/{loader}/profile/json"))?;
    if fabric["inheritsFrom"].as_str()!=Some(mc){return Err("Pack Fabric Loader is not available for its Minecraft version".into())}
    let profile=json!({"id":uuid::Uuid::new_v4().to_string(),"name":strv(&manifest,"name")?,"version":mc,"ram":4096,"java":"","fabric_loader":loader,"modpack":manifest["versionId"]});
    let final_dir=profile_dir(&profile)?;
    let parent=final_dir.parent().ok_or("Invalid profile directory")?;let stage=stage(parent)?;
    let dest=stage.0.join("profile");fs::create_dir_all(&dest).map_err(err)?;
    let files=manifest["files"].as_array().ok_or("Missing modpack file list")?;
    if files.len()>10000{return Err("Too many modpack files".into())}
    let mut seen=HashSet::new();
    for (i,f) in files.iter().enumerate(){
        if f["env"]["client"]=="unsupported"||f["env"]["client"]=="optional"{continue}
        let rel=strv(f,"path")?;let p=allowed_path(rel)?;
        if !seen.insert(rel.to_ascii_lowercase()){return Err(format!("Duplicate modpack path {rel}"))}
        let hash=strv(&f["hashes"],"sha512")?;
        let urls=f["downloads"].as_array().ok_or("Missing modpack downloads")?;
        let out=dest.join(p);
        let mut failure=Some("No supported download URL".to_owned());
        log(app,format!("Downloading pack file {}/{}: {}",i+1,files.len(),rel));
        let _=app.emit("launcher-progress",((i+1)*85/files.len().max(1)) as u32);
        for url in urls.iter().filter_map(Value::as_str){
            match download(&c,url,hash,f["fileSize"].as_u64(),&out){Ok(())=>{failure=None;break},Err(e)=>failure=Some(e)}
        }
        if let Some(e)=failure{return Err(format!("{rel}: {e}"))}
    }
    let mut total=0u64;
    for prefix in ["overrides/","client-overrides/"]{
        for i in 0..archive.len(){
            let mut f=archive.by_index(i).map_err(err)?;
            let name=f.name().to_owned();let Some(rel)=name.strip_prefix(prefix)else{continue};
            if f.is_dir(){continue}
            if f.unix_mode().is_some_and(|m|m&0o170000==0o120000){return Err("Symbolic links in modpacks are not supported".into())}
            let p=allowed_path(rel)?;
            total=total.checked_add(f.size()).ok_or("Archive exceeds size limit")?;
            if total>2*LIMIT||f.size()>LIMIT{return Err("Pack overrides exceed size limit".into())}
            let out=dest.join(p);fs::create_dir_all(out.parent().ok_or("Invalid override path")?).map_err(err)?;
            std::io::copy(&mut f,&mut fs::File::create(out).map_err(err)?).map_err(err)?;
        }
    }
    fs::write(dest.join("fabric-profile.json"),serde_json::to_vec_pretty(&fabric).map_err(err)?).map_err(err)?;
    fs::write(dest.join(".rylune-pack.json"),serde_json::to_vec_pretty(&manifest).map_err(err)?).map_err(err)?;
    fs::rename(&dest,&final_dir).map_err(err)?;
    Ok(profile)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn pack_paths(){for s in ["../mods/a.jar","mods/../a.jar","mods/C:/a.jar","mods/a.jar:evil","mods/CON.jar","runtime/bin/java.exe","settings.json","mods/a. "]{assert!(allowed_path(s).is_err(),"{s}");}assert!(allowed_path("mods/sodium.jar").is_ok());assert!(allowed_path("config/sodium-options.json").is_ok());}
    #[test]fn extensions(){assert!(ext_ok("mod","a.jar"));assert!(!ext_ok("mod","a.exe"));assert!(ext_ok("resourcepack","a.zip"));}

    #[test]
    fn primary_download_selection(){
        let single=json!({"files":[{"filename":"only.jar"}]});
        assert_eq!(primary(&single).unwrap()["filename"],"only.jar");
        let multiple=json!({"files":[{"filename":"extra.jar"},{"filename":"main.jar","primary":true}]});
        assert_eq!(primary(&multiple).unwrap()["filename"],"main.jar");
        let ambiguous=json!({"files":[{"filename":"a.jar"},{"filename":"b.jar"}]});
        assert_eq!(primary(&ambiguous).unwrap_err(),"No unambiguous primary download");
    }
}
