// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod tray;

use tauri::{Emitter, Manager, Url};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_opener::OpenerExt; // 用于调用 app.emit

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--autostart"])))
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .max_file_size(50_000 /* bytes */)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: Some("logs".to_string()) }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            log::info!("single instance triggered: {args:?}, cwd: {cwd}");
            // 如果 args[1] 以 opener 开头，则解析 url，获得域名和路径以及参数
            if args.len() > 1 && args[1].starts_with("opener://") {
                // let url_str = args[1].replace("opener://", "");

                // 使用 match 来处理错误，避免 unwrap() 导致 panic
                match Url::parse(&args[1]) {
                    Ok(url) => {
                        if url.scheme() == "opener" {
                            app.emit("opener", url.to_string()).unwrap();
                            if let Some(query_pairs) =
                                url.query_pairs().find(|(key, _)| key == "path")
                            {
                                let path = query_pairs.1.to_string();
                                if let Err(e) = app.opener().open_path(&path, None::<&str>) {
                                    log::error!("Failed to open path: {}", e);
                                } else {
                                    log::info!("Opening path: {}", path);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse URL: {}", e);
                    }
                }
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // 注册自定义协议
            // #[cfg(any(target_os = "linux", target_os = "windows"))]
            // app.deep_link().register_all()?;

            // 禁用右键菜单
            let window = app.get_webview_window("main").unwrap();
            window.eval("document.addEventListener('contextmenu', event => event.preventDefault());").unwrap();

            println!("Setting up URL handler...");
            let args = std::env::args().collect::<Vec<_>>();
            // 如果 args[1] 以 opener 开头，则解析 url，获得域名和路径以及参数
            if args.len() > 1 && args[1].starts_with("opener://") {
                // let url_str = args[1].replace("opener://", "");

                // 使用 match 来处理错误，避免 unwrap() 导致 panic
                match Url::parse(&args[1]) {
                    Ok(url) => {
                        if url.scheme() == "opener" {
                            app.emit("opener", url.to_string()).unwrap();
                            if let Some(query_pairs) =
                                url.query_pairs().find(|(key, _)| key == "path")
                            {
                                let path = query_pairs.1.to_string();
                                if let Err(e) = app.opener().open_path(&path, None::<&str>) {
                                    log::error!("Failed to open path: {}", e);
                                } else {
                                    log::info!("Opening path: {}", path);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse URL: {}", e);
                    }
                }
            }

            // 设置托盘事件处理
            tray::create_tray(app.handle())?;

            // 设置窗口关闭事件处理
            let window = app.get_webview_window("main").unwrap();
            window.clone().on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    window.hide().unwrap();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
