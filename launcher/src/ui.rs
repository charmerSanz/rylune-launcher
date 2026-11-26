use super::*;
use eframe::egui::*;
use std::{collections::VecDeque, time::{Duration,Instant}};
const BG:Color32=Color32::from_rgb(2,2,3);
const SIDEBAR:Color32=Color32::from_rgb(5,5,6);
const GREEN:Color32=Color32::from_rgb(110,202,155);
const PANEL:Color32=Color32::from_rgb(10,10,12);
const LINE:Color32=Color32::from_rgb(32,32,36);
const MUTED:Color32=Color32::from_rgb(165,165,174);
const RED:Color32=Color32::from_rgb(221,35,56);
const HOT:Color32=Color32::from_rgb(255,96,112);
const WHITE:Color32=Color32::from_rgb(245,245,247);
fn field<'a>(v:&'a Value,key:&str)->&'a str{v[key].as_str().unwrap_or("")}
fn items(v:&Value,key:&str)->Vec<Value>{v[key].as_array().cloned().unwrap_or_default()}
fn text(s:impl Into<String>,size:f32,color:Color32)->RichText{RichText::new(s).size(size).color(color)}
// A real egui button supplies focus, disabled state and keyboard activation.
// Its background is drawn behind the standard label with cached, inexpensive shapes.
fn accent(ui:&mut Ui,label:&str,w:f32,h:f32,enabled:bool)->bool{
    let painter=ui.painter().clone();
    let layer=painter.add(Shape::Noop);
    let response=ui.add_enabled(enabled,Button::new(text(label,16.0,WHITE).strong())
        .fill(Color32::TRANSPARENT).stroke(Stroke::NONE).min_size(vec2(w,h)));
    let hover=ui.ctx().animate_bool(response.id.with("premium_hover"),enabled&&(response.hovered()||response.has_focus()));
    let press=ui.ctx().animate_bool(response.id.with("premium_press"),enabled&&response.is_pointer_button_down_on());
    let r=response.rect.shrink(1.0);
    let radius=(r.height()*0.5).min(28.0);
    let fill=if enabled{blend(Color32::from_rgb(173,13,35),Color32::from_rgb(240,45,67),hover*0.72)}else{Color32::from_rgb(42,17,22)};
    let edge=if enabled{blend(Color32::from_rgb(209,40,62),HOT,hover)}else{Color32::from_rgb(72,28,36)};
    let mut shapes=Vec::with_capacity(9);
    if enabled{
        for i in (1..=4).rev(){
            let d=i as f32*1.5;
            shapes.push(Shape::rect_filled(r.expand(d).translate(vec2(0.0,2.0)),radius+d,Color32::from_rgba_unmultiplied(208,20,45,((5.0+hover*7.0)*(5-i) as f32) as u8)));
        }
        shapes.push(Shape::rect_filled(r.translate(vec2(0.0,3.0*(1.0-press))),radius,Color32::from_rgb(59,5,15)));
    }
    shapes.push(Shape::rect_filled(r,radius,fill));
    shapes.push(Shape::rect_filled(Rect::from_min_max(r.min+vec2(2.0,2.0),pos2(r.right()-2.0,r.center().y+2.0)),radius,if enabled{blend(Color32::from_rgb(201,28,50),HOT,hover*0.35)}else{Color32::from_rgb(51,20,27)}));
    shapes.push(Shape::rect_stroke(r,radius,Stroke::new(1.0_f32,edge),StrokeKind::Inside));
    painter.set(layer,Shape::Vec(shapes));
    response.clicked()
}
fn surface()->Frame{
    Frame::new().fill(PANEL).stroke(Stroke::new(1.0_f32,LINE)).corner_radius(18).inner_margin(Margin::same(18))
}
fn blend(a:Color32,b:Color32,t:f32)->Color32{
    let t=t.clamp(0.0,1.0);
    let f=|x:u8,y:u8|(x as f32+(y as f32-x as f32)*t).round() as u8;
    Color32::from_rgb(f(a.r(),b.r()),f(a.g(),b.g()),f(a.b(),b.b()))
}
fn nav_item(ui:&mut Ui,p:Page,active:bool,reduce:bool)->bool{
    let (r,res)=ui.allocate_exact_size(vec2(ui.available_width(),42.0),Sense::click());
    let hovered=res.hovered();
    let a=if reduce{if hovered{1.0}else{0.0}}else{ui.ctx().animate_bool(res.id.with("hover"),hovered)};
    let selected=if reduce{if active{1.0}else{0.0}}else{ui.ctx().animate_bool(res.id.with("active"),active)};
    let fill=blend(blend(SIDEBAR,Color32::from_rgb(18,18,21),a),Color32::from_rgb(63,10,20),selected);
    ui.painter().rect_filled(r,21.0,fill);
    if selected>0.01{ui.painter().rect_stroke(r,21.0,Stroke::new(1.0_f32,blend(LINE,RED,selected)),StrokeKind::Inside);ui.painter().rect_filled(Rect::from_min_size(r.left_top()+vec2(3.0,8.0),vec2(3.0,26.0*selected)),2.0,HOT);}
    let color=blend(MUTED,WHITE,selected.max(a));
    let c=r.left_center()+vec2(21.0,0.0);
    let st=Stroke::new(1.5_f32,color);
    match p{
        Page::Home=>{ui.painter().line_segment([c+vec2(-7.0,0.0),c+vec2(0.0,-7.0)],st);ui.painter().line_segment([c+vec2(0.0,-7.0),c+vec2(7.0,0.0)],st);ui.painter().rect_stroke(Rect::from_center_size(c+vec2(0.0,3.0),vec2(11.0,10.0)),1.0,st,StrokeKind::Inside);}
        Page::Downloads=>{ui.painter().line_segment([c+vec2(0.0,-7.0),c+vec2(0.0,5.0)],st);ui.painter().line_segment([c+vec2(-5.0,0.0),c+vec2(0.0,5.0)],st);ui.painter().line_segment([c+vec2(5.0,0.0),c+vec2(0.0,5.0)],st);}
        Page::Accounts=>{ui.painter().circle_stroke(c+vec2(0.0,-4.0),3.5,st);ui.painter().rect_stroke(Rect::from_center_size(c+vec2(0.0,6.0),vec2(13.0,7.0)),3.0,st,StrokeKind::Inside);}
        Page::Settings=>{ui.painter().circle_stroke(c,7.0,st);ui.painter().circle_stroke(c,2.5,st);}
        _=>{ui.painter().rect_stroke(Rect::from_center_size(c,vec2(15.0,13.0)),2.0,st,StrokeKind::Inside);ui.painter().line_segment([c+vec2(-4.0,-2.0),c+vec2(4.0,-2.0)],st);ui.painter().line_segment([c+vec2(-4.0,2.0),c+vec2(4.0,2.0)],st);}
    }
    ui.painter().text(r.left_center()+vec2(41.0+a*2.0,0.0),Align2::LEFT_CENTER,p.label(),FontId::proportional(14.0),color);
    res.clicked()
}
fn panel()->Frame{surface()}
fn texture(ctx:&Context,name:&str,bytes:&[u8])->Option<TextureHandle>{
    let i=image::load_from_memory(bytes).ok()?.into_rgba8();
    let sz=[i.width() as usize,i.height() as usize];
    Some(ctx.load_texture(name,ColorImage::from_rgba_unmultiplied(sz,i.as_raw()),TextureOptions::LINEAR))
}
// Rounded textured mesh: true clipping at corners, with tint interpolated by the GPU.
fn wallpaper(ui:&Ui,r:Rect,t:&Option<TextureHandle>,hero:bool,hover:f32){
    let radius=if hero{22.0_f32}else{17.0_f32};
    ui.painter().rect_filled(r,radius,PANEL);
    if let Some(t)=t{
        let a=t.size_vec2().x/t.size_vec2().y;let target=r.width()/r.height().max(1.0);
        let uv=if a>target{let w=target/a;Rect::from_min_max(pos2((1.0-w)/2.0,0.0),pos2((1.0+w)/2.0,1.0))}else{let h=a/target;Rect::from_min_max(pos2(0.0,(1.0-h)/2.0),pos2(1.0,(1.0+h)/2.0))};
        let mut mesh=eframe::egui::epaint::Mesh::with_texture(t.id());
        let vertex=|p:Pos2|{
            let x=((p.x-r.left())/r.width().max(1.0)).clamp(0.0,1.0);
            let y=((p.y-r.top())/r.height().max(1.0)).clamp(0.0,1.0);
            let v=if hero{(35.0+190.0*x-22.0*y).clamp(15.0,235.0)}else{(155.0-98.0*y+hover*36.0).clamp(30.0,210.0)};
            eframe::egui::epaint::Vertex{pos:p,uv:pos2(uv.left()+uv.width()*x,uv.top()+uv.height()*y),color:Color32::from_gray(v as u8)}
        };
        mesh.vertices.push(vertex(r.center()));
        let corners=[(pos2(r.right()-radius,r.top()+radius),-90.0_f32),(pos2(r.right()-radius,r.bottom()-radius),0.0),(pos2(r.left()+radius,r.bottom()-radius),90.0),(pos2(r.left()+radius,r.top()+radius),180.0)];
        for (center,start) in corners{
            for i in 0..=8{
                let angle=(start+90.0*i as f32/8.0).to_radians();
                mesh.vertices.push(vertex(center+vec2(angle.cos(),angle.sin())*radius));
            }
        }
        let n=mesh.vertices.len() as u32;
        for i in 1..n{mesh.indices.extend_from_slice(&[0,i,if i+1==n{1}else{i+1}]);}
        ui.painter().add(Shape::mesh(mesh));
    }
}
fn startup_overlay(ctx:&Context,logo:&Option<TextureHandle>,elapsed:f32,cover_only:bool){
    let r=ctx.screen_rect();
    let p=ctx.layer_painter(LayerId::new(Order::Foreground,Id::new("startup_reveal")));
    let fade=if cover_only{(1.0-((elapsed-0.72)/0.38).clamp(0.0,1.0)).powi(2)}else{1.0};
    p.rect_filled(r,0.0,Color32::from_black_alpha((255.0*fade) as u8));
    if cover_only{return}
    let enter=1.0-(1.0-(elapsed/0.48).clamp(0.0,1.0)).powi(3);
    let c=r.center()+vec2(0.0,-22.0+(1.0-enter)*15.0);
    let size=72.0+24.0*enter;
    for i in (1..=6).rev(){p.circle_filled(c,size*0.5+i as f32*8.0,Color32::from_rgba_unmultiplied(180,12,36,(4.0*enter) as u8));}
    if let Some(t)=logo{
        p.image(t.id(),Rect::from_center_size(c,vec2(size,size)),Rect::from_min_max(Pos2::ZERO,pos2(1.0,1.0)),Color32::from_white_alpha((255.0*enter) as u8));
    }
    p.text(c+vec2(0.0,76.0),Align2::CENTER_CENTER,"R Y L U N E",FontId::proportional(25.0),Color32::from_white_alpha((255.0*enter) as u8));
    let track=Rect::from_center_size(c+vec2(0.0,111.0),vec2(122.0,3.0));
    p.rect_filled(track,2.0,Color32::from_rgb(24,14,17));
    p.rect_filled(Rect::from_min_size(track.min,vec2(track.width()*(elapsed/0.72).clamp(0.0,1.0),track.height())),2.0,RED);
}
#[derive(Clone,Copy,PartialEq,Eq)]
enum Page{Home,Versions,Profiles,Downloads,Accounts,Settings,Logs}
impl Page{fn label(self)->&'static str{match self{Self::Home=>"Home",Self::Versions=>"Versions",Self::Profiles=>"Profiles",Self::Downloads=>"Downloads",Self::Accounts=>"Accounts",Self::Settings=>"Settings",Self::Logs=>"Logs"}}}
struct Draft{id:String,name:String,version:String,ram:u64,java:String}
pub struct Launcher{
    cfg:Value, app:AppHandle, rx:mpsc::Receiver<LauncherEvent>, page:Page,
    busy:bool, running:bool, progress:f32, status:String, error:String,
    logs:VecDeque<String>, versions:Vec<String>, fetching:bool, search:String, installed:Vec<String>,
    offline:String, client_id:String, login:bool, device:Option<Value>, draft:Option<Draft>, confirm_remove:Option<String>,
    hero:Option<TextureHandle>, card:Option<TextureHandle>, logo:Option<TextureHandle>, reduce_motion:bool,
    last_page:Page, page_since:Instant, installed_only:bool, log_search:String, install_task:bool, startup:Option<Instant>,
}
impl Launcher{
    pub fn new(cc:&eframe::CreationContext<'_>)->Self{
        let ctx=&cc.egui_ctx;let mut style=(*ctx.style()).clone();
        style.visuals=Visuals::dark();
        style.visuals.panel_fill=BG;style.visuals.window_fill=PANEL;style.visuals.extreme_bg_color=BG;
        style.visuals.override_text_color=Some(WHITE);
        style.visuals.widgets.inactive.bg_fill=Color32::from_rgb(17,17,19);
        style.visuals.widgets.hovered.bg_fill=Color32::from_rgb(32,23,26);
        style.visuals.widgets.active.bg_fill=Color32::from_rgb(68,17,26);
        style.visuals.widgets.inactive.weak_bg_fill=Color32::from_rgb(17,17,19);
        style.visuals.widgets.hovered.weak_bg_fill=Color32::from_rgb(32,23,26);
        style.visuals.widgets.active.weak_bg_fill=Color32::from_rgb(68,17,26);
        for widget in [&mut style.visuals.widgets.inactive,&mut style.visuals.widgets.hovered,&mut style.visuals.widgets.active,&mut style.visuals.widgets.open]{widget.corner_radius=CornerRadius::same(18);widget.bg_stroke=Stroke::new(1.0_f32,LINE);}
        style.visuals.window_corner_radius=CornerRadius::same(20);
        style.visuals.selection.bg_fill=RED;style.visuals.selection.stroke=Stroke::new(1.0_f32,WHITE);
        style.spacing.item_spacing=vec2(12.0,12.0);style.spacing.button_padding=vec2(17.0,10.0);
        style.animation_time=0.23;style.spacing.interact_size=vec2(40.0,34.0);ctx.set_style(style);
        let (tx,rx)=mpsc::channel();
        let app=AppHandle{runtime:Arc::new(Runtime{tokens:Mutex::new(HashMap::new()),devices:Mutex::new(HashMap::new())}),tx,ctx:ctx.clone()};
        let (cfg,error)=match load_config(){Ok(c)=>(c,String::new()),Err(e)=>(json!({"profiles":[{"id":RYLUNE,"name":"Minecraft 1.21.11","version":RYLUNE,"ram":2048,"java":""}],"accounts":[],"profile":RYLUNE,"account":"","client_id":""}),format!("Settings could not be loaded: {e}"))};
        let mut s=Self{client_id:field(&cfg,"client_id").to_owned(),reduce_motion:cfg["reduce_motion"].as_bool().unwrap_or(false),cfg,app,rx,page:Page::Home,last_page:Page::Home,page_since:Instant::now(),installed_only:false,log_search:String::new(),install_task:false,startup:None,busy:false,running:false,progress:0.0,status:"Ready".into(),error,logs:VecDeque::new(),versions:vec![RYLUNE.into()],fetching:false,search:String::new(),installed:installed_versions().unwrap_or_default(),offline:String::new(),login:false,device:None,draft:None,confirm_remove:None,hero:texture(ctx,"hero",include_bytes!("../public/bg-hero.png")),card:texture(ctx,"card",include_bytes!("../public/bg-card.png")),logo:texture(ctx,"logo",include_bytes!("../public/logo.png"))};
        if s.reduce_motion{ctx.style_mut(|style|style.animation_time=0.0);}s.catalog();s
    }
    fn catalog(&mut self){if self.fetching{return}self.fetching=true;let a=self.app.clone();std::thread::spawn(move||a.send(LauncherEvent::Catalog(fetch_catalog())));}
    fn save(&mut self,next:Value)->bool{match save_config(next.clone()){Ok(())=>{self.cfg=next;true},Err(e)=>{self.error=e;false}}}
    fn patch(&mut self,k:&str,v:Value){let mut n=self.cfg.clone();n[k]=v;self.save(n);}
    fn active_profile(&self)->Value{let list=items(&self.cfg,"profiles");list.iter().find(|p|field(p,"id")==field(&self.cfg,"profile")).cloned().or_else(||list.first().cloned()).unwrap_or(Value::Null)}
    fn active_account(&self)->Value{items(&self.cfg,"accounts").into_iter().find(|a|field(a,"id")==field(&self.cfg,"account")).unwrap_or(Value::Null)}
    fn select_version(&mut self,v:&str)->Option<Value>{
        if !supported_version(v){self.error="Unsupported Minecraft version".into();return None}
        let mut n=self.cfg.clone();let mut list=items(&n,"profiles");
        let p=if let Some(p)=list.iter().find(|p|field(p,"version")==v){p.clone()}else{let p=json!({"id":v,"name":format!("Minecraft {v}"),"version":v,"ram":2048,"java":""});list.push(p.clone());p};
        n["profiles"]=json!(list);n["profile"]=p["id"].clone();if self.save(n){Some(p)}else{None}
    }
    fn run(&mut self,install:bool,version:Option<String>){
        if self.busy{return}
        let account=self.active_account();
        if !install&&account.is_null(){self.page=Page::Accounts;self.error="Select an account to play.".into();return}
        let profile=if let Some(v)=version{let Some(p)=self.select_version(&v)else{return};p}else{self.active_profile()};
        if profile.is_null(){self.error="Select a profile.".into();return}
        self.busy=true;self.running=false;self.install_task=install;self.progress=0.0;self.error.clear();self.status=if install{"Installing".into()}else{"Preparing Minecraft".into()};
        let a=self.app.clone();std::thread::spawn(move||{
            let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||game::run(&a,&profile,&account,install))).unwrap_or_else(|_|Err("Game worker failed".into()));
            a.send(LauncherEvent::Finished(result,install));
        });
    }
    fn add_account(&mut self,a:Value){let mut n=self.cfg.clone();let mut accounts=items(&n,"accounts");accounts.retain(|v|v["id"]!=a["id"]);accounts.push(a.clone());n["accounts"]=json!(accounts);n["account"]=a["id"].clone();self.save(n);}
    fn sign_in(&mut self){
        if self.login{return}if self.client_id.trim().is_empty(){self.page=Page::Settings;self.error="Set the Microsoft application ID.".into();return}
        self.login=true;self.error.clear();self.device=None;
        let a=self.app.clone();let id=self.client_id.trim().to_owned();
        std::thread::spawn(move||{
            let result=(||->R<Value>{
                let d=auth::start(&a,&id)?;a.send(LauncherEvent::Device(d.clone()));
                let session=field(&d,"session").to_owned();let mut interval=d["interval"].as_u64().unwrap_or(5).max(5);
                loop{std::thread::sleep(Duration::from_secs(interval));let r=auth::poll(&a,&session)?;if r["pending"]==true{interval=r["interval"].as_u64().unwrap_or(interval).max(5);continue}return Ok(r["account"].clone())}
            })();a.send(LauncherEvent::Login(result));
        });
    }
    fn events(&mut self){
        while let Ok(e)=self.rx.try_recv(){match e{
            LauncherEvent::Log(l)=>{if !self.running&&(l.starts_with("Assets ")||l.starts_with("Java ")||l.starts_with("Downloading")||l.starts_with("Installing")||l.starts_with("Checking")||l.starts_with("Preparing")){self.status=l.clone();}if self.logs.len()>=2500{self.logs.pop_front();}self.logs.push_back(l);}
            LauncherEvent::Progress(n)=>self.progress=n as f32/100.0,
            LauncherEvent::State(s)=>{if s=="running"{self.running=true;self.status="Minecraft is running".into();}},
            LauncherEvent::Finished(res,install)=>{self.busy=false;self.running=false;self.installed=installed_versions().unwrap_or_default();match res{Ok(())=>{self.status=if install{"Installed".into()}else{"Game closed".into()};self.progress=1.0},Err(e)=>{self.status="Failed".into();self.error=e.clone();self.logs.push_back(e);self.page=Page::Logs}}}
            LauncherEvent::Catalog(res)=>{self.fetching=false;match res{Ok(v)=>self.versions=v,Err(e)=>{self.logs.push_back(format!("Version catalog: {e}"));}}}
            LauncherEvent::Device(v)=>self.device=Some(v),
            LauncherEvent::Login(res)=>{self.login=false;self.device=None;match res{Ok(a)=>self.add_account(a),Err(e)=>self.error=e}}
        }}
    }
    fn profile_installed(&self,p:&Value)->bool{profile_dir(p).map(|d|d.join(".rylune-installed.json").is_file()&&d.join("versions").join(field(p,"version")).join(format!("{}.jar",field(p,"version"))).is_file()).unwrap_or(false)}
    fn play_label(&self)->&'static str{if self.running{"Running"}else if self.busy{if self.install_task{"Installing…"}else{"Starting…"}}else if self.profile_installed(&self.active_profile()){"Play"}else{"Install & play"}}
    fn top(&mut self,ctx:&Context){
        TopBottomPanel::top("header").frame(Frame::new().fill(SIDEBAR).inner_margin(Margin::symmetric(24,12))).show(ctx,|ui|{
            ui.horizontal(|ui|{
                if let Some(t)=&self.logo{ui.add(Image::new((t.id(),vec2(42.0,42.0))));}
                ui.vertical(|ui|{ui.label(text("RYLUNE",25.0,WHITE).strong());ui.label(text("C L I E N T",10.0,MUTED));});ui.add_space(14.0);ui.label(text(self.page.label(),13.0,MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                    let account=self.active_account();let name=field(&account,"name");
                    if ui.button(if name.is_empty(){"Add account"}else{name}).clicked(){self.page=Page::Accounts;}
                    if self.busy&&ui.button(if self.running{"Game running"}else{"View download"}).clicked(){self.page=if self.running{Page::Logs}else{Page::Downloads};}
                });
            });
        });
    }
    fn sidebar(&mut self,ctx:&Context){
        SidePanel::left("sidebar").exact_width(208.0).resizable(false).frame(Frame::new().fill(SIDEBAR).inner_margin(Margin::same(14))).show(ctx,|ui|{
            ui.add_space(12.0);
            for (label,pages) in [("PLAY",&[Page::Home,Page::Versions,Page::Profiles,Page::Downloads][..]),("MANAGE",&[Page::Accounts,Page::Settings,Page::Logs][..])]{
                ui.label(text(label,10.0,MUTED).strong());ui.add_space(2.0);
                for p in pages{if nav_item(ui,*p,self.page==*p,self.reduce_motion){self.page=*p;}}
                ui.add_space(18.0);
            }
            ui.with_layout(Layout::bottom_up(Align::Min),|ui|{
                ui.label(text(concat!("v",env!("CARGO_PKG_VERSION")),11.0,MUTED));
                let p=self.active_profile();ui.label(text(format!("{} Fabric",field(&p,"version")),13.0,WHITE).strong());
                ui.label(text("ACTIVE PROFILE",10.0,MUTED));ui.separator();
            });
        });
    }
    fn bottom(&mut self,ctx:&Context){
        TopBottomPanel::bottom("launch").frame(Frame::new().fill(BG).inner_margin(Margin::symmetric(14,10))).show(ctx,|ui|{
            surface().inner_margin(Margin::symmetric(18,10)).show(ui,|ui|{ui.horizontal(|ui|{
                if let Some(t)=&self.logo{ui.add(Image::new((t.id(),vec2(44.0,44.0))));}
                let p=self.active_profile();
                ui.vertical(|ui|{
                    ui.label(text("ACTIVE PROFILE",10.0,MUTED));
                    let mut select=None;
                    ComboBox::from_id_salt("active_profile").width(205.0).selected_text(field(&p,"name")).show_ui(ui,|ui|{for v in items(&self.cfg,"profiles"){if ui.selectable_label(v["id"]==p["id"],field(&v,"name")).clicked(){select=Some(v["id"].clone())}}});
                    if let Some(v)=select{if !self.busy{self.patch("profile",v);}}
                    ui.label(text(format!("{}  /  {} MB",field(&p,"version"),p["ram"].as_u64().unwrap_or(2048)),11.0,MUTED));
                });
                ui.add_space(12.0);
                let w=(ui.available_width()-196.0).max(80.0);
                ui.allocate_ui_with_layout(vec2(w,58.0),Layout::top_down(Align::Max),|ui|{
                    ui.horizontal(|ui|{let (r,_)=ui.allocate_exact_size(vec2(8.0,8.0),Sense::hover());ui.painter().circle_filled(r.center(),3.0,if self.running{GREEN}else{HOT});ui.label(text(if self.busy&&!self.running{format!("{}  ·  {:.0}%",self.status,self.progress*100.0)}else{self.status.clone()},12.0,MUTED));});
                    let progress=if self.busy{self.progress}else{0.0};let shown=if self.reduce_motion{progress}else{ctx.animate_value_with_time(Id::new("download_progress"),progress,0.18)};
                    ui.add(ProgressBar::new(shown).desired_width(w).desired_height(5.0).fill(RED));
                });
                if accent(ui,self.play_label(),176.0,52.0,!self.busy){self.run(false,None);}
            });});
        });
    }
    fn home(&mut self,ui:&mut Ui){
        let w=ui.available_width();let h=(ui.available_height()-290.0).clamp(245.0,440.0);
        let (rect,_)=ui.allocate_exact_size(vec2(w,h),Sense::hover());wallpaper(ui,rect,&self.hero,true,0.0);
        ui.painter().rect_stroke(rect,22.0,Stroke::new(1.0_f32,LINE),StrokeKind::Inside);
        let p=self.active_profile();
        let inner=Rect::from_min_max(rect.min+vec2(38.0,0.0),rect.max-vec2(32.0,0.0));
        ui.allocate_new_ui(UiBuilder::new().max_rect(inner),|ui|{ui.vertical(|ui|{
            ui.add_space((h*0.16).max(12.0));ui.label(text("FABRIC",13.0,Color32::from_rgb(247,125,139)).strong());
            ui.label(text(format!("{} Fabric",field(&p,"version")),if w>850.0{54.0}else{39.0},WHITE).strong());
            ui.label(text("Minecraft: Java Edition",16.0,MUTED));ui.add_space(10.0);
            ui.horizontal(|ui|{if accent(ui,self.play_label(),190.0,54.0,!self.busy){self.run(false,None);}if ui.add_enabled(!self.busy,Button::new("Configure").min_size(vec2(110.0,50.0))).clicked(){self.edit(Some(p.clone()));}});
        });});
        ui.add_space(10.0);
        let account=self.active_account();
        let tiles=[("Profile",field(&p,"name").to_owned(),Page::Profiles),("Account",if account.is_null(){"Not selected".into()}else{field(&account,"name").to_owned()},Page::Accounts),("Installation",if self.profile_installed(&p){"Installed".into()}else{"Not installed".into()},Page::Downloads)];
        ui.horizontal(|ui|{
            for (label,value,page) in tiles{
                let (r,res)=ui.allocate_exact_size(vec2(((w-24.0)/3.0).max(100.0),75.0),Sense::click());
                let a=if self.reduce_motion{if res.hovered(){1.0}else{0.0}}else{ui.ctx().animate_bool(res.id,res.hovered())};
                ui.painter().rect_filled(r,17.0,blend(PANEL,Color32::from_rgb(24,15,19),a));
                ui.painter().rect_stroke(r,17.0,Stroke::new(1.0_f32,blend(LINE,RED,a*0.5)),StrokeKind::Inside);
                ui.painter().circle_filled(r.left_center()+vec2(18.0,0.0),4.0,blend(RED,HOT,a));
                ui.painter().text(r.left_top()+vec2(34.0,12.0),Align2::LEFT_TOP,label,FontId::proportional(12.0),MUTED);
                let name=if value.chars().count()>20{format!("{}…",value.chars().take(19).collect::<String>())}else{value};
                ui.painter().text(r.left_bottom()+vec2(34.0,-15.0),Align2::LEFT_BOTTOM,name,FontId::proportional(15.0),WHITE);
                if res.clicked(){self.page=page;}
            }
        });
        ui.add_space(10.0);ui.horizontal(|ui|{ui.label(text("Versions",22.0,WHITE).strong());ui.with_layout(Layout::right_to_left(Align::Center),|ui|{if ui.button("All versions").clicked(){self.page=Page::Versions;}});});
        let mut cards:Vec<String>=["1.21.11","26.2","26.3"].into_iter().filter(|v|self.versions.iter().any(|x|x==v)).map(str::to_owned).collect();
        for v in &self.versions{if cards.len()==3{break}if !cards.contains(v){cards.push(v.clone())}}
        let cw=((w-24.0)/3.0).max(100.0);
        ui.horizontal(|ui|{for v in cards{
            let (r,response)=ui.allocate_exact_size(vec2(cw,128.0),Sense::click());
            let a=if self.reduce_motion{if response.hovered(){1.0}else{0.0}}else{ui.ctx().animate_bool(response.id,response.hovered())};
            let r=r.translate(vec2(0.0,if self.reduce_motion{0.0}else{-3.0*a}));
            let selected=field(&p,"version")==v;
            if selected||a>0.01{ui.painter().rect_filled(r.expand(3.0),19.0,Color32::from_rgba_unmultiplied(225,25,55,if selected{38}else{(24.0*a) as u8}));}
            wallpaper(ui,r,&self.card,false,a);
            ui.painter().rect_stroke(r,17.0,Stroke::new(if selected{2.0_f32}else{1.0_f32},if selected{HOT}else{blend(LINE,RED,a*0.65)}),StrokeKind::Inside);
            if selected{let c=r.right_top()+vec2(-19.0,19.0);ui.painter().circle_filled(c,10.0,Color32::from_rgb(66,10,22));ui.painter().circle_stroke(c,10.0,Stroke::new(1.0_f32,HOT));ui.painter().line_segment([c+vec2(-4.0,0.0),c+vec2(-1.0,3.0)],Stroke::new(1.6_f32,WHITE));ui.painter().line_segment([c+vec2(-1.0,3.0),c+vec2(5.0,-4.0)],Stroke::new(1.6_f32,WHITE));}
            ui.painter().text(r.left_bottom()+vec2(16.0,-36.0),Align2::LEFT_BOTTOM,format!("{v} Fabric"),FontId::proportional(23.0),WHITE);
            ui.painter().text(r.left_bottom()+vec2(16.0,-14.0),Align2::LEFT_BOTTOM,"Minecraft: Java Edition",FontId::proportional(12.0),MUTED);
            if response.clicked()&&!self.busy{self.select_version(&v);}
        }});
    }
    fn downloads(&mut self,ui:&mut Ui){
        let p=self.active_profile();
        panel().show(ui,|ui|{
            ui.label(text(if self.running{"Minecraft is running"}else if self.busy{"Current task"}else{"Installation"},23.0,WHITE).strong());
            ui.label(text(field(&p,"name"),13.0,MUTED));ui.add_space(6.0);
            ui.horizontal(|ui|{ui.label(text(self.status.clone(),16.0,WHITE));ui.with_layout(Layout::right_to_left(Align::Center),|ui|{ui.label(text(format!("{:.0}%",self.progress*100.0),22.0,WHITE).strong());});});
            let progress=if self.reduce_motion{self.progress}else{ui.ctx().animate_value_with_time(Id::new("task_progress"),self.progress,0.3)};
            ui.add(ProgressBar::new(progress).desired_width(ui.available_width()).desired_height(9.0).fill(RED));ui.add_space(8.0);
            ui.horizontal(|ui|{
                if accent(ui,if self.profile_installed(&p){"Repair"}else{"Install"},115.0,41.0,!self.busy){self.run(true,None);}
                if ui.button("Open folder").clicked(){if let Err(e)=open_folder(Some(field(&p,"id").to_owned())){self.error=e;}}
                if ui.button("View logs").clicked(){self.page=Page::Logs;}
            });
        });
        ui.label(text("Activity",22.0,WHITE).strong());
        panel().show(ui,|ui|{if self.logs.is_empty(){ui.label(text("No activity yet.",13.0,MUTED));}for l in self.logs.iter().rev().take(8){ui.label(text(l.clone(),12.0,MUTED));}});
    }
    fn versions(&mut self,ui:&mut Ui){
        ui.horizontal(|ui|{ui.add(TextEdit::singleline(&mut self.search).hint_text("Search versions").desired_width(240.0));ui.checkbox(&mut self.installed_only,"Installed only");if ui.add_enabled(!self.fetching,Button::new("Refresh")).clicked(){self.catalog();}});
        ui.label(text(format!("{} available  /  {} installed",self.versions.len(),self.installed.len()),12.0,MUTED));
        for v in self.versions.clone(){if !v.contains(&self.search)||(self.installed_only&&!self.installed.contains(&v)){continue}
            panel().show(ui,|ui|{ui.horizontal(|ui|{
                ui.vertical(|ui|{ui.label(text(format!("{v} Fabric"),19.0,WHITE).strong());ui.label(text(if self.installed.contains(&v){"Installed"}else{"Minecraft: Java Edition"},12.0,MUTED));});
                ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                    if accent(ui,if self.installed.contains(&v){"Repair"}else{"Install"},100.0,38.0,!self.busy){self.run(true,Some(v.clone()));}
                    if ui.add_enabled(!self.busy,Button::new("Select")).clicked(){self.select_version(&v);}
                });
            });});
        }
    }
    fn edit(&mut self,p:Option<Value>){
        self.draft=Some(if let Some(p)=p{Draft{id:field(&p,"id").into(),name:field(&p,"name").into(),version:field(&p,"version").into(),ram:p["ram"].as_u64().unwrap_or(2048),java:field(&p,"java").into()}}else{Draft{id:String::new(),name:String::new(),version:RYLUNE.into(),ram:2048,java:String::new()}})
    }
    fn profiles(&mut self,ui:&mut Ui){
        if accent(ui,"Create profile",145.0,40.0,!self.busy){self.edit(None);}
        let profiles=items(&self.cfg,"profiles");
        for p in profiles.clone(){panel().show(ui,|ui|{
            ui.horizontal(|ui|{ui.label(text(field(&p,"name"),20.0,WHITE).strong());if p["id"]==self.cfg["profile"]{ui.label(text("Selected",12.0,GREEN));}});
            ui.label(text(format!("{} Fabric  ·  {} MB  ·  {}",field(&p,"version"),p["ram"].as_u64().unwrap_or(2048),if self.profile_installed(&p){"Installed"}else{"Not installed"}),13.0,MUTED));
            ui.horizontal(|ui|{
                if ui.add_enabled(!self.busy,Button::new("Select")).clicked(){self.patch("profile",p["id"].clone());}
                if ui.add_enabled(!self.busy,Button::new("Edit")).clicked(){self.edit(Some(p.clone()));}
                if ui.button("Open folder").clicked(){if let Err(e)=open_folder(Some(field(&p,"id").to_owned())){self.error=e}}
                if ui.add_enabled(!self.busy&&profiles.len()>1,Button::new("Remove")).clicked(){self.confirm_remove=Some(field(&p,"id").to_owned());}
            });
        });}
    }
    fn accounts(&mut self,ui:&mut Ui){
        panel().show(ui,|ui|{
            ui.label(text("Offline account",20.0,WHITE).strong());
            ui.label(text("Local play for Minecraft owners.",13.0,MUTED));
            ui.horizontal(|ui|{ui.add(TextEdit::singleline(&mut self.offline).hint_text("Username").char_limit(16));if accent(ui,"Add account",120.0,38.0,!self.busy){match offline_account(self.offline.trim().to_owned()){Ok(a)=>{self.add_account(a);self.offline.clear()},Err(e)=>self.error=e}}});
        });
        panel().show(ui,|ui|{
            ui.label(text("Microsoft account",20.0,WHITE).strong());
            if accent(ui,if self.login{"Signing in…"}else{"Sign in"},120.0,40.0,!self.login&&!self.busy){self.sign_in();}
            if let Some(d)=&self.device{ui.label("Enter this code on the Microsoft sign-in page:");ui.label(text(field(d,"code"),28.0,WHITE).strong());ui.horizontal(|ui|{if ui.button("Copy code").clicked(){ui.ctx().copy_text(field(d,"code").to_owned());}if ui.button("Open sign-in page").clicked(){let _=open::that(field(d,"url"));}});}
        });
        ui.label(text("Saved accounts",23.0,WHITE).strong());
        let active=field(&self.cfg,"account").to_owned();
        for a in items(&self.cfg,"accounts"){panel().show(ui,|ui|{
            ui.horizontal(|ui|{ui.vertical(|ui|{
                ui.label(text(field(&a,"name"),19.0,WHITE).strong());
                let signed=signed_in(&self.app.runtime);
                ui.label(text(format!("{}{}",field(&a,"type"),if a["type"]=="Microsoft"&&!signed.contains(&field(&a,"id").to_owned()){" · Sign-in required"}else{""}),12.0,MUTED));
            });ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                if ui.add_enabled(!self.busy,Button::new("Remove")).clicked(){let mut n=self.cfg.clone();let mut list=items(&n,"accounts");list.retain(|x|x["id"]!=a["id"]);n["accounts"]=json!(list);if n["account"]==a["id"]{n["account"]=list.first().map(|x|x["id"].clone()).unwrap_or(json!(""));}if self.save(n){self.app.runtime.tokens.lock().unwrap().remove(field(&a,"id"));}}
                if ui.add_enabled(!self.busy,Button::new(if field(&a,"id")==active{"Selected"}else{"Select"})).clicked(){self.patch("account",a["id"].clone());}
            });});
        });}
    }
    fn settings(&mut self,ui:&mut Ui){
        panel().show(ui,|ui|{
            ui.label(text("Microsoft authentication",20.0,WHITE).strong());ui.label(text("Application ID",13.0,MUTED));
            ui.add(TextEdit::singleline(&mut self.client_id).desired_width(360.0));
            if ui.button("Save").clicked(){self.patch("client_id",json!(self.client_id.trim()));}
        });
        panel().show(ui,|ui|{
            ui.label(text("Launcher",20.0,WHITE).strong());
            if ui.checkbox(&mut self.reduce_motion,"Reduce animations").changed(){self.patch("reduce_motion",json!(self.reduce_motion));ui.ctx().style_mut(|s|s.animation_time=if self.reduce_motion{0.0}else{0.23});}
            if ui.button("Open data directory").clicked(){if let Err(e)=open_folder(None){self.error=e;}}
            ui.label(text(concat!("Version ",env!("CARGO_PKG_VERSION")),12.0,MUTED));
        });
    }
    fn logs(&mut self,ui:&mut Ui){
        ui.horizontal(|ui|{ui.add(TextEdit::singleline(&mut self.log_search).hint_text("Search logs").desired_width(220.0));if ui.button("Copy logs").clicked(){ui.ctx().copy_text(self.logs.iter().cloned().collect::<Vec<_>>().join("\n"));}if ui.button("Clear").clicked(){self.logs.clear();}});
        ScrollArea::vertical().stick_to_bottom(true).show(ui,|ui|{for l in &self.logs{if !self.log_search.is_empty()&&!l.to_lowercase().contains(&self.log_search.to_lowercase()){continue}ui.label(RichText::new(l).monospace().size(12.0).color(if l.to_lowercase().contains("error"){Color32::from_rgb(255,140,145)}else{MUTED}));}});
    }
    fn dialogs(&mut self,ctx:&Context){
        if let Some(mut d)=self.draft.take(){
            let mut close=false;let mut save=false;
            Window::new("Configure profile").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER,vec2(0.0,0.0)).show(ctx,|ui|{
                ui.label("Profile name");ui.text_edit_singleline(&mut d.name);
                ui.label("Minecraft version");ComboBox::from_id_salt("edit_version").selected_text(&d.version).show_ui(ui,|ui|{let mut versions=self.versions.clone();if !versions.contains(&d.version){versions.push(d.version.clone())}for v in versions{ui.selectable_value(&mut d.version,v.clone(),v);}});
                ui.label("RAM (MB)");ui.add(DragValue::new(&mut d.ram).range(512..=65536).speed(128));
                ui.horizontal(|ui|{for ram in [1024_u64,2048,4096,6144]{if ui.selectable_label(d.ram==ram,format!("{} GB",ram/1024)).clicked(){d.ram=ram;}}});
                ui.label("Java executable");ui.add(TextEdit::singleline(&mut d.java).hint_text("Automatic"));
                ui.horizontal(|ui|{if accent(ui,"Save profile",130.0,38.0,true){save=true;}if ui.button("Cancel").clicked(){close=true;}});
            });
            if save{if d.name.trim().is_empty()||!supported_version(&d.version){self.error="Enter a profile name and valid version.".into()}else{
                let mut n=self.cfg.clone();let mut list=items(&n,"profiles");
                let id=if d.id.is_empty(){if list.iter().any(|p|field(p,"id")==d.version||field(p,"version")==d.version){uuid::Uuid::new_v4().to_string()}else{d.version.clone()}}else{d.id.clone()};
                let v=json!({"id":id,"name":d.name.trim(),"version":d.version,"ram":d.ram,"java":d.java.trim()});
                list.retain(|p|p["id"]!=v["id"]);list.push(v.clone());n["profiles"]=json!(list);n["profile"]=v["id"].clone();close=self.save(n);
            }}
            if !close{self.draft=Some(d);}
        }
        if let Some(id)=self.confirm_remove.clone(){
            let mut done=false;
            Window::new("Remove profile?").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER,Vec2::ZERO).show(ctx,|ui|{
                ui.label("Game files will be kept.");ui.horizontal(|ui|{
                    if accent(ui,"Remove",100.0,36.0,true){let mut n=self.cfg.clone();let mut list=items(&n,"profiles");list.retain(|p|field(p,"id")!=id);if !list.is_empty(){n["profiles"]=json!(list);if field(&n,"profile")==id{n["profile"]=list[0]["id"].clone();}self.save(n);}done=true;}
                    if ui.button("Cancel").clicked(){done=true;}
                });
            });if done{self.confirm_remove=None;}
        }
    }
}
impl eframe::App for Launcher{
    fn update(&mut self,ctx:&Context,_:&mut eframe::Frame){
        self.events();
        let elapsed=self.startup.get_or_insert_with(Instant::now).elapsed().as_secs_f32();
        if !self.reduce_motion&&elapsed<0.72{
            CentralPanel::default().frame(Frame::new().fill(BG)).show(ctx,|_|{});
            startup_overlay(ctx,&self.logo,elapsed,false);ctx.request_repaint_after(Duration::from_millis(16));return;
        }
        self.top(ctx);self.bottom(ctx);self.sidebar(ctx);
        if self.page!=self.last_page{self.last_page=self.page;self.page_since=Instant::now();}
        let reveal=if self.reduce_motion{1.0}else{1.0-(1.0-(self.page_since.elapsed().as_secs_f32()/0.24).clamp(0.0,1.0)).powi(3)};
        if reveal<0.999{ctx.request_repaint_after(Duration::from_millis(16));}
        CentralPanel::default().frame(Frame::new().fill(BG).inner_margin(Margin::same(22))).show(ctx,|ui|{
            ui.add_space((1.0-reveal)*9.0);
            if !self.error.is_empty(){ui.horizontal(|ui|{ui.label(text(self.error.clone(),13.0,Color32::from_rgb(255,130,145)));if ui.button("Dismiss").clicked(){self.error.clear();}});}
            if self.page!=Page::Home{ui.label(text(self.page.label(),34.0,WHITE).strong());ui.add_space(8.0);}
            if self.page==Page::Logs{self.logs(ui);}else{ScrollArea::vertical().id_salt(self.page.label()).auto_shrink([false,false]).show(ui,|ui|{match self.page{Page::Home=>self.home(ui),Page::Versions=>self.versions(ui),Page::Profiles=>self.profiles(ui),Page::Downloads=>self.downloads(ui),Page::Accounts=>self.accounts(ui),Page::Settings=>self.settings(ui),Page::Logs=>{}}});}
            if reveal<0.999{ui.painter().rect_filled(ui.max_rect(),0.0,Color32::from_rgba_unmultiplied(BG.r(),BG.g(),BG.b(),((1.0-reveal)*95.0) as u8));}
        });
        self.dialogs(ctx);
        if !self.reduce_motion&&elapsed<1.10{startup_overlay(ctx,&self.logo,elapsed,true);ctx.request_repaint_after(Duration::from_millis(16));}
    }
}
