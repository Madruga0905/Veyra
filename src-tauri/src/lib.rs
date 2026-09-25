use tauri::{Emitter, Manager, WebviewUrl, webview::{DownloadEvent, NewWindowResponse, WebviewBuilder}, PhysicalPosition, PhysicalSize};
use std::{collections::HashMap,io::Write,sync::{Arc,Mutex,OnceLock},sync::atomic::{AtomicBool,AtomicU64,Ordering}};
use serde_json::json;
use tauri_plugin_updater::UpdaterExt;
#[cfg(windows)] use webview2_com::{AcceleratorKeyPressedEventHandler,WebResourceRequestedEventHandler,take_pwstr};
#[cfg(windows)] use webview2_com::Microsoft::Web::WebView2::Win32::{COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN,COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL};
#[cfg(windows)] use windows::core::{HSTRING,PWSTR};
#[cfg(windows)] #[link(name="user32")] extern "system" { fn GetKeyState(n_virt_key:i32)->i16; }
const TOP: u32 = 118;
fn data_root()->std::path::PathBuf{let base=std::env::var("LOCALAPPDATA").unwrap_or_else(|_|std::env::temp_dir().to_string_lossy().to_string());std::path::PathBuf::from(base).join("Veyra")}
fn log_path()->std::path::PathBuf{data_root().join("veyra.log")}
fn extensions_root()->std::path::PathBuf{data_root().join("Extensions")}
fn proton_manifest()->std::path::PathBuf{extensions_root().join("proton-pass").join("manifest.json")}
fn log_line(message:&str){let path=log_path();if let Some(parent)=path.parent(){let _=std::fs::create_dir_all(parent);}if let Ok(mut f)=std::fs::OpenOptions::new().create(true).append(true).open(path){let _=writeln!(f,"{}",message);let _=f.flush();}}
fn safe_name(value:&str)->String{let s=value.chars().filter(|c|c.is_ascii_alphanumeric()||*c=='-'||*c=='_').collect::<String>();if s.is_empty(){"extension".into()}else{s}}
fn copy_dir_all(src:&std::path::Path,dst:&std::path::Path)->std::io::Result<()>{std::fs::create_dir_all(dst)?;for entry in std::fs::read_dir(src)?{let entry=entry?;let ty=entry.file_type()?;let target=dst.join(entry.file_name());if ty.is_dir(){copy_dir_all(&entry.path(),&target)?;}else{std::fs::copy(entry.path(),target)?;}}Ok(())}
fn manifest_version(path:&std::path::Path)->Option<String>{let raw=std::fs::read_to_string(path.join("manifest.json")).ok()?;serde_json::from_str::<serde_json::Value>(&raw).ok()?.get("version")?.as_str().map(str::to_string)}
fn ensure_bundled_extensions(app:&tauri::AppHandle)->Result<(),String>{let root=extensions_root();std::fs::create_dir_all(&root).map_err(|e|e.to_string())?;let packaged=app.path().resource_dir().map_err(|e|e.to_string())?.join("resources").join("proton-pass");let dev=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources").join("proton-pass");let src=if packaged.is_dir(){packaged}else{dev};if !src.is_dir(){return Err("Proton Pass bundled resource not found".into());}let dst=root.join("proton-pass");if !dst.is_dir()||manifest_version(&src)!=manifest_version(&dst){if dst.exists(){std::fs::remove_dir_all(&dst).map_err(|e|e.to_string())?;}copy_dir_all(&src,&dst).map_err(|e|e.to_string())?;}Ok(())}
fn safe_profile(profile_id:&str)->String{profile_id.chars().filter(|c|c.is_ascii_alphanumeric()||*c=='-'||*c=='_').collect::<String>()}
fn label(profile_id:&str,tab_id:u32)->String{format!("content-{}-{tab_id}",safe_profile(profile_id))}
fn profile_data_dir(profile_id:&str)->std::path::PathBuf{data_root().join("Profiles").join(safe_profile(profile_id))}
fn each_content(app:&tauri::AppHandle)->Vec<tauri::Webview>{app.webviews().into_iter().filter(|(k,_)|k.starts_with("content-")).map(|(_,v)|v).collect()}
fn profile_content(app:&tauri::AppHandle,profile_id:&str)->Vec<tauri::Webview>{let p=format!("content-{}-",safe_profile(profile_id));app.webviews().into_iter().filter(|(k,_)|k.starts_with(&p)).map(|(_,v)|v).collect()}
struct AdblockState{enabled:AtomicBool,blocked:AtomicU64}
static ADBLOCK_STATES:OnceLock<Mutex<HashMap<String,Arc<AdblockState>>>>=OnceLock::new();
const AD_HOSTS:&[&str]=&["doubleclick.net","googlesyndication.com","googleadservices.com","googletagservices.com","amazon-adsystem.com","adnxs.com","rubiconproject.com","pubmatic.com","openx.net","casalemedia.com","criteo.com","criteo.net","adsrvr.org","taboola.com","outbrain.com","scorecardresearch.com","quantserve.com","moatads.com","media.net","smartadserver.com","sharethrough.com","bidswitch.net","lijit.com","bluekai.com","demdex.net","serving-sys.com","adsafeprotected.com","adform.net","yieldmo.com","zedo.com"];
fn adblock_state(profile_id:&str)->Arc<AdblockState>{let m=ADBLOCK_STATES.get_or_init(||Mutex::new(HashMap::new()));let mut g=m.lock().unwrap_or_else(|e|e.into_inner());g.entry(safe_profile(profile_id)).or_insert_with(||Arc::new(AdblockState{enabled:AtomicBool::new(true),blocked:AtomicU64::new(0)})).clone()}
fn should_block_url(url:&str)->bool{let Ok(u)=tauri::Url::parse(url)else{return false;};let Some(host)=u.host_str()else{return false;};let h=host.to_ascii_lowercase();AD_HOSTS.iter().any(|d|h==*d||h.ends_with(&format!(".{d}")))}
fn hide_all(app:&tauri::AppHandle){for w in each_content(app){let _=w.hide();}}
fn resize_content(app:&tauri::AppHandle){
 if let Some(window)=app.get_window("main"){
  if let Ok(size)=window.inner_size(){for w in each_content(app){let _=w.set_position(PhysicalPosition::new(0,TOP as i32));let _=w.set_size(PhysicalSize::new(size.width,size.height.saturating_sub(TOP)));}}
 }
}
#[cfg(windows)]
fn attach_adblock(webview:&tauri::Webview,app:tauri::AppHandle,profile_id:String,tab_id:u32){
 let state=adblock_state(&profile_id);let _=webview.with_webview(move|wv|{let controller=wv.controller();let env=wv.environment();let event_app=app.clone();let event_profile=profile_id.clone();
  let Ok(core)= (unsafe{controller.CoreWebView2()}) else{return;};unsafe{let _=core.AddWebResourceRequestedFilter(&HSTRING::from("*"),COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL);}
  let handler=WebResourceRequestedEventHandler::create(Box::new(move|_sender,args|{if !state.enabled.load(Ordering::Relaxed){return Ok(());}let Some(args)=args else{return Ok(());};unsafe{let req=args.Request()?;let mut raw=PWSTR::null();req.Uri(&mut raw)?;let url=take_pwstr(raw);if should_block_url(&url){let status=HSTRING::from("No Content");let headers=HSTRING::from("Cache-Control: no-store\r\n");let resp=env.CreateWebResourceResponse(None,204,&status,&headers)?;args.SetResponse(&resp)?;let total=state.blocked.fetch_add(1,Ordering::Relaxed)+1;let _=event_app.emit("veyra-adblock",json!({"profileId":event_profile,"tabId":tab_id,"url":url,"total":total}));log_line(&format!("ADBLOCK profile={} tab={} total={} {}",event_profile,tab_id,total,url));}}Ok(())}));let mut token=0i64;unsafe{let _=core.add_WebResourceRequested(&handler,&mut token);}
 });
}
#[cfg(not(windows))] fn attach_adblock(_webview:&tauri::Webview,_app:tauri::AppHandle,_profile_id:String,_tab_id:u32){}
#[cfg(windows)]
fn attach_shortcuts(webview:&tauri::Webview,app:tauri::AppHandle,profile_id:String,tab_id:u32){
 let _=webview.with_webview(move|wv|{let controller=wv.controller();let event_app=app.clone();let event_profile=profile_id.clone();
  let handler=AcceleratorKeyPressedEventHandler::create(Box::new(move|_sender,args|{
   if let Some(args)=args{let mut key=0u32;let mut kind=COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN;unsafe{args.VirtualKey(&mut key)?;args.KeyEventKind(&mut kind)?;}
    if kind.0==COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN.0||kind.0==COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN.0{
     let ctrl=unsafe{GetKeyState(0x11)<0};let alt=unsafe{GetKeyState(0x12)<0};let shift=unsafe{GetKeyState(0x10)<0};
     let action=if ctrl{match key{0x54=>Some("new-tab"),0x57=>Some("close-tab"),0x4C=>Some("focus-address"),0x52=>Some("reload"),0x48=>Some("history"),0x4A=>Some("downloads"),0x09=>Some(if shift{"prev-tab"}else{"next-tab"}),_=>None}}else if alt{match key{0x25=>Some("back"),0x27=>Some("forward"),_=>None}}else{None};
     if let Some(action)=action{if matches!(action,"new-tab"|"focus-address"|"history"|"downloads"){if let Some(shell)=event_app.get_webview("main"){let _=shell.set_focus();}}let _=event_app.emit("veyra-accelerator",json!({"action":action,"tabId":tab_id,"profileId":event_profile}));unsafe{args.SetHandled(true)?;}}
    }
   }Ok(())
  }));let mut token=0i64;unsafe{let _=controller.add_AcceleratorKeyPressed(&handler,&mut token);}
 });
}
#[cfg(not(windows))] fn attach_shortcuts(_webview:&tauri::Webview,_app:tauri::AppHandle,_profile_id:String,_tab_id:u32){}
fn navigate_impl(app:tauri::AppHandle,profile_id:String,tab_id:u32,url:String,extensions_enabled:bool,adblock_enabled:bool)->Result<(),String>{
 adblock_state(&profile_id).enabled.store(adblock_enabled,Ordering::Relaxed);
 log_line(&format!("NAVIGATE profile={profile_id} tab={tab_id} adblock={adblock_enabled} {url}"));
 let parsed:tauri::Url=url.parse().map_err(|e|format!("URL invalido: {e}"))?;
 if !matches!(parsed.scheme(),"http"|"https"){return Err("Apenas URLs http/https são permitidos".into());}
 hide_all(&app); let web_label=label(&profile_id,tab_id);
 if let Some(webview)=app.get_webview(&web_label){webview.navigate(parsed).map_err(|e|e.to_string())?;webview.show().map_err(|e|e.to_string())?;let _=webview.set_focus();resize_content(&app);return Ok(());}
 let window=app.get_window("main").ok_or("janela principal indisponivel")?; let size=window.inner_size().map_err(|e|e.to_string())?;
 let page_id=tab_id;let title_id=tab_id;let download_id=tab_id;let popup_id=tab_id;let popup_app=app.clone();
 let page_profile=profile_id.clone();let title_profile=profile_id.clone();let download_profile=profile_id.clone();let popup_profile=profile_id.clone();
 let mut builder=WebviewBuilder::new(web_label.clone(),WebviewUrl::External(parsed)).data_directory(profile_data_dir(&profile_id));
 if extensions_enabled{builder=builder.browser_extensions_enabled(true).extensions_path(extensions_root());}
 let builder=builder.on_new_window(move|url,_features|{let _=popup_app.emit("veyra-new-window",json!({"profileId":popup_profile,"sourceTabId":popup_id,"url":url.to_string()}));NewWindowResponse::Deny})
  .on_page_load(move|webview,payload|{let url=payload.url().to_string();log_line(&format!("PAGE profile={} tab={} {:?} {}",page_profile,page_id,payload.event(),url));let _=webview.app_handle().emit("veyra-page",json!({"profileId":page_profile,"tabId":page_id,"url":url,"event":format!("{:?}",payload.event())}));})
  .on_document_title_changed(move|webview,title|{let _=webview.app_handle().emit("veyra-title",json!({"profileId":title_profile,"tabId":title_id,"title":title}));})
  .on_download(move|webview,event|{match event{
   DownloadEvent::Requested{url,destination}=>{let _=webview.app_handle().emit("veyra-download",json!({"profileId":download_profile,"tabId":download_id,"url":url.to_string(),"path":destination.to_string_lossy(),"name":destination.file_name().map(|x|x.to_string_lossy().to_string()).unwrap_or_default(),"finished":false}));}
   DownloadEvent::Finished{url,path,success}=>{let _=webview.app_handle().emit("veyra-download",json!({"profileId":download_profile,"tabId":download_id,"url":url.to_string(),"path":path.as_ref().map(|p|p.to_string_lossy().to_string()).unwrap_or_default(),"name":path.as_ref().and_then(|p|p.file_name()).map(|x|x.to_string_lossy().to_string()).unwrap_or_default(),"finished":true,"success":success}));}_=>{}}true});
 let child=window.add_child(builder,PhysicalPosition::new(0,TOP as i32),PhysicalSize::new(size.width,size.height.saturating_sub(TOP))).map_err(|e|e.to_string())?;
 attach_adblock(&child,app.clone(),profile_id.clone(),tab_id);attach_shortcuts(&child,app.clone(),profile_id.clone(),tab_id); let _=child.set_focus();
 log_line(&format!("CHILD profile={profile_id} tab={tab_id} label={web_label} url={:?}",child.url())); Ok(())
}
#[tauri::command] async fn navigate(app:tauri::AppHandle,profile_id:String,tab_id:u32,url:String,extensions_enabled:bool,adblock_enabled:bool)->Result<(),String>{tauri::async_runtime::spawn_blocking(move||navigate_impl(app,profile_id,tab_id,url,extensions_enabled,adblock_enabled)).await.map_err(|e|e.to_string())?}
#[tauri::command] fn set_adblock_enabled(app:tauri::AppHandle,profile_id:String,enabled:bool)->Result<(),String>{let s=adblock_state(&profile_id);s.enabled.store(enabled,Ordering::Relaxed);for w in profile_content(&app,&profile_id){let _=w.reload();}log_line(&format!("ADBLOCK_TOGGLE profile={} enabled={}",profile_id,enabled));Ok(())}
#[tauri::command] fn adblock_info(profile_id:String)->serde_json::Value{let s=adblock_state(&profile_id);json!({"enabled":s.enabled.load(Ordering::Relaxed),"blocked":s.blocked.load(Ordering::Relaxed),"rules":AD_HOSTS.len()})}
#[tauri::command] fn switch_tab(app:tauri::AppHandle,profile_id:String,tab_id:u32)->Result<(),String>{hide_all(&app);if let Some(w)=app.get_webview(&label(&profile_id,tab_id)){w.show().map_err(|e|e.to_string())?;let _=w.set_focus();resize_content(&app);}Ok(())}
#[tauri::command] fn close_tab(app:tauri::AppHandle,profile_id:String,tab_id:u32)->Result<(),String>{if let Some(w)=app.get_webview(&label(&profile_id,tab_id)){w.close().map_err(|e|e.to_string())?;}Ok(())}
#[tauri::command] fn hide_browser(app:tauri::AppHandle){hide_all(&app);if let Some(shell)=app.get_webview("main"){let _=shell.set_focus();}}
#[tauri::command] fn browser_back(app:tauri::AppHandle,profile_id:String,tab_id:u32){if let Some(w)=app.get_webview(&label(&profile_id,tab_id)){let _=w.eval("history.back()");}}
#[tauri::command] fn browser_forward(app:tauri::AppHandle,profile_id:String,tab_id:u32){if let Some(w)=app.get_webview(&label(&profile_id,tab_id)){let _=w.eval("history.forward()");}}
#[tauri::command] fn browser_reload(app:tauri::AppHandle,profile_id:String,tab_id:u32){if let Some(w)=app.get_webview(&label(&profile_id,tab_id)){let _=w.reload();}}
#[tauri::command] fn window_minimize(app:tauri::AppHandle)->Result<(),String>{log_line("WINDOW minimize");app.get_window("main").ok_or("janela principal indisponivel")?.minimize().map_err(|e|e.to_string())}
#[tauri::command] fn window_toggle_maximize(app:tauri::AppHandle)->Result<(),String>{let w=app.get_window("main").ok_or("janela principal indisponivel")?;let max=w.is_maximized().map_err(|e|e.to_string())?;log_line(&format!("WINDOW toggle_maximize currently={max}"));if max{w.unmaximize().map_err(|e|e.to_string())}else{w.maximize().map_err(|e|e.to_string())}}
#[tauri::command] fn window_start_dragging(app:tauri::AppHandle)->Result<(),String>{app.get_window("main").ok_or("janela principal indisponivel")?.start_dragging().map_err(|e|e.to_string())}
#[tauri::command] fn window_close(app:tauri::AppHandle)->Result<(),String>{log_line("WINDOW close");app.get_window("main").ok_or("janela principal indisponivel")?.close().map_err(|e|e.to_string())}
#[tauri::command] fn clear_browser_data(app:tauri::AppHandle,profile_id:String)->Result<(),String>{for w in profile_content(&app,&profile_id){w.clear_all_browsing_data().map_err(|e|e.to_string())?;}Ok(())}
#[tauri::command] fn close_profile(app:tauri::AppHandle,profile_id:String)->Result<(),String>{for w in profile_content(&app,&profile_id){let _=w.close();}Ok(())}
#[tauri::command] fn delete_profile_data(app:tauri::AppHandle,profile_id:String)->Result<(),String>{for w in profile_content(&app,&profile_id){let _=w.close();}let dir=profile_data_dir(&profile_id);if dir.exists(){std::fs::remove_dir_all(dir).map_err(|e|e.to_string())?;}Ok(())}
#[tauri::command] fn open_downloads_folder()->Result<(),String>{let home=std::env::var("USERPROFILE").map_err(|e|e.to_string())?;std::process::Command::new("explorer.exe").arg(format!("{}\\Downloads",home)).spawn().map_err(|e|e.to_string())?;Ok(())}
#[tauri::command] fn reveal_download(path:String)->Result<(),String>{let p=std::path::PathBuf::from(path);if !p.exists(){return Err("O ficheiro já não existe neste caminho".into());}std::process::Command::new("explorer.exe").arg("/select,").arg(&p).spawn().map_err(|e|e.to_string())?;Ok(())}
#[tauri::command] fn open_extensions_folder()->Result<(),String>{let root=extensions_root();std::fs::create_dir_all(&root).map_err(|e|e.to_string())?;std::process::Command::new("explorer.exe").arg(root).spawn().map_err(|e|e.to_string())?;Ok(())}
#[tauri::command] fn list_extensions()->serde_json::Value{let mut out=Vec::new();if let Ok(entries)=std::fs::read_dir(extensions_root()){for entry in entries.flatten(){let p=entry.path();if !p.is_dir(){continue;}let manifest=p.join("manifest.json");let Ok(raw)=std::fs::read_to_string(&manifest)else{continue;};let Ok(v)=serde_json::from_str::<serde_json::Value>(&raw)else{continue;};out.push(json!({"folder":entry.file_name().to_string_lossy(),"name":v.get("name").and_then(|x|x.as_str()).unwrap_or("Extensão Chromium"),"version":v.get("version").and_then(|x|x.as_str()).unwrap_or("?"),"manifestVersion":v.get("manifest_version").and_then(|x|x.as_u64()).unwrap_or(0),"protected":entry.file_name().to_string_lossy()=="proton-pass"}));}}json!(out)}
#[tauri::command] fn install_extension_folder(source_path:String)->Result<serde_json::Value,String>{let src=std::path::PathBuf::from(source_path.trim().trim_matches('"'));let manifest=src.join("manifest.json");if !src.is_dir()||!manifest.is_file(){return Err("Seleciona uma pasta de extensão Chromium que contenha manifest.json".into());}let raw=std::fs::read_to_string(&manifest).map_err(|e|e.to_string())?;let v:serde_json::Value=serde_json::from_str(&raw).map_err(|_|"manifest.json inválido".to_string())?;let display=v.get("name").and_then(|x|x.as_str()).unwrap_or("extension");let base=safe_name(&src.file_name().map(|x|x.to_string_lossy().to_string()).unwrap_or_else(||display.to_string()));let root=extensions_root();std::fs::create_dir_all(&root).map_err(|e|e.to_string())?;let mut folder=base.clone();let mut n=2;while root.join(&folder).exists(){folder=format!("{base}-{n}");n+=1;}let dst=root.join(&folder);copy_dir_all(&src,&dst).map_err(|e|e.to_string())?;Ok(json!({"folder":folder,"name":display,"version":v.get("version").and_then(|x|x.as_str()).unwrap_or("?"),"restartRequired":true}))}
#[tauri::command] fn remove_extension(folder:String)->Result<(),String>{if folder=="proton-pass"{return Err("O Proton Pass está protegido no Veyra.".into());}let name=safe_name(&folder);let path=extensions_root().join(name);if path.exists(){std::fs::remove_dir_all(path).map_err(|e|e.to_string())?;}Ok(())}
#[tauri::command]
fn extension_info(profile_id:String)->serde_json::Value{
 let raw=std::fs::read_to_string(proton_manifest()).ok();
 let parsed=raw.as_deref().and_then(|s|serde_json::from_str::<serde_json::Value>(s).ok());
 let root=profile_data_dir(&profile_id);let candidates=[root.join("Default").join("Secure Preferences"),root.join("EBWebView").join("Default").join("Secure Preferences")];
 let installed=candidates.iter().filter_map(|p|std::fs::read_to_string(p).ok()).any(|s|s.contains("ghmbeldphafepmbegfdlkpapadhbakde"));
 json!({"ready":parsed.is_some(),"installed":installed,"name":parsed.as_ref().and_then(|v|v.get("name")).and_then(|v|v.as_str()).unwrap_or("Proton Pass"),"version":parsed.as_ref().and_then(|v|v.get("version")).and_then(|v|v.as_str()).unwrap_or("?"),"id":"ghmbeldphafepmbegfdlkpapadhbakde"})
}
#[tauri::command] fn app_version(app:tauri::AppHandle)->String{app.package_info().version.to_string()}
#[tauri::command] async fn check_for_update(app:tauri::AppHandle)->Result<serde_json::Value,String>{let current=app.package_info().version.to_string();let updater=app.updater().map_err(|e|e.to_string())?;match updater.check().await.map_err(|e|e.to_string())?{Some(update)=>Ok(json!({"available":true,"currentVersion":current,"version":update.version.to_string(),"date":update.date.map(|d|d.to_string()),"body":update.body})),None=>Ok(json!({"available":false,"currentVersion":current}))}}
#[tauri::command] async fn install_update(app:tauri::AppHandle)->Result<(),String>{let updater=app.updater().map_err(|e|e.to_string())?;let update=updater.check().await.map_err(|e|e.to_string())?.ok_or("Nenhuma atualização disponível")?;update.download_and_install(|_,_|{},||{}).await.map_err(|e|e.to_string())?;#[cfg(not(target_os="windows"))] app.restart();#[allow(unreachable_code)] Ok(())}
#[cfg_attr(mobile,tauri::mobile_entry_point)]
pub fn run(){
 tauri::Builder::default()
 .plugin(tauri_plugin_updater::Builder::new().build())
 .setup(|app|{let _=std::fs::create_dir_all(data_root());let _=std::fs::write(log_path(),"SETUP start extensions=enabled real-tabs=enabled updater=enabled\n");if let Err(e)=ensure_bundled_extensions(app.handle()){log_line(&format!("EXTENSION_SETUP_ERROR {e}"));}Ok(())})
 .on_window_event(|window,event|if matches!(event,tauri::WindowEvent::Resized(_)){resize_content(&window.app_handle());})
 .invoke_handler(tauri::generate_handler![navigate,set_adblock_enabled,adblock_info,switch_tab,close_tab,close_profile,delete_profile_data,hide_browser,browser_back,browser_forward,browser_reload,window_minimize,window_toggle_maximize,window_start_dragging,window_close,clear_browser_data,open_downloads_folder,reveal_download,open_extensions_folder,list_extensions,install_extension_folder,remove_extension,extension_info,app_version,check_for_update,install_update])
 .run(tauri::generate_context!()).expect("erro ao iniciar Veyra");
}
