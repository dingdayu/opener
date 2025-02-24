// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod core;
mod macros;

use tauri::{Manager, Url};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;
use tokio::time::{sleep, Duration}; // ✅ 正确导入 sleep

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // 日志插件应优先初始化
        .plugin(
            tauri_plugin_log::Builder::new()
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .max_file_size(50_000 /* bytes */)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .build(),
        )
        // 处理单例模式，防止多个实例运行
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            log::info!("single instance triggered: {:?}", args);
            if let Some(url) = args.get(1) {
                if url.starts_with("opener://") {
                    handle_opener_url(url);
                }
            } else {
                log::info!("no args found, showing window");
                // 如果没 args1 且 窗口处于隐藏状态，则显示窗口
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(false) = window.is_visible() {
                        window.show().unwrap();
                    }
                }
            }
        }))
        // 其他插件
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        // 统一的 setup 处理
        .setup(|app| {
            log::info!("app setup started");

            core::handle::Handle::global().init(app.app_handle());

            // 1. 注册 deep link 监听（核心改动）
            // #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            // {
            //     if let Err(e) = app.deep_link().register_all() {
            //         log::error!("Failed to register deep link: {:?}", e);
            //     }
            // }

            app.deep_link().on_open_url(move |event| {
                tauri::async_runtime::spawn(async move {
                    if let Some(url) = event.urls().first() {
                        log::info!("Deep Link Received via on_open_url: {}", url);
                        if url.as_str().starts_with("opener://") {
                            handle_opener_url(url.as_str());
                        }
                    }
                });
            });

            // 2. 创建系统托盘
            core::tray::create_tray(app.handle())?;
            log::info!("tray setup complete");

            // 3. 处理窗口事件
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();

                // 监听窗口关闭事件 -> 最小化到托盘
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        window_clone.hide().unwrap();
                    }
                });

                // 禁用右键菜单
                window
                    .eval("document.addEventListener('contextmenu', event => event.preventDefault());")
                    .unwrap();

                log::info!("window event setup complete");

                // 🚀 **启动后 500ms 后隐藏窗口**
                tauri::async_runtime::spawn(async move {
                    sleep(Duration::from_millis(500)).await;
                    if let Err(e) = window.hide() {
                        log::error!("Failed to hide main window on startup: {}", e);
                    }
                });
            } else {
                log::error!("Failed to get main window instance");
            }

            // 5. 解析命令行参数（应用首次启动时处理 deep link）
            let args = std::env::args().collect::<Vec<_>>();
            log::info!("app start args: {args:?}");
            if let Some(url) = args.get(1) {
                if url.starts_with("opener://") {
                    handle_opener_url(url);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 处理 opener:// URL
fn handle_opener_url(url: &str) {
    match Url::parse(url) {
        Ok(parsed_url) => {
            if parsed_url.scheme() == "opener" {
                log::info!("Processing deep link: {}", parsed_url);

                let app_handle = core::handle::Handle::global().app_handle().unwrap();
                // app_handle.emit("opener", parsed_url.to_string()).unwrap();

                // 解析 URL 参数 path
                if let Some(query_pairs) = parsed_url.query_pairs().find(|(key, _)| key == "path") {
                    let path = query_pairs.1.to_string();

                    if !path.is_empty() && std::path::Path::new(&path).exists() {
                        log::info!("Open ready:  {}", path);
                        if let Err(e) = app_handle.opener().open_path(&path, None::<&str>) {
                            log::error!("Failed to open path: {}", e);
                        } else {
                            log::info!("Opening path: {}", path);
                        }
                    } else {
                        log::error!("Invalid path: {}", path);
                    }
                }

                // // 🚀 **Deep Link 触发时显示窗口**
                // if let Some(window) = app_handle.get_webview_window("main") {
                //     if let Err(e) = window.show() {
                //         log::error!("Failed to show main window: {}", e);
                //     }
                //     if let Err(e) = window.set_focus() {
                //         log::error!("Failed to focus main window: {}", e);
                //     }
                // }
            }
        }
        Err(e) => {
            log::error!("Failed to parse URL: {}", e);
        }
    }
}
