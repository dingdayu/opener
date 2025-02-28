// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod core;
mod macros;

use tauri::{Manager, Url};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;
use tokio::time::{sleep, Duration}; // ✅ 正确导入 sleep
use reqwest;

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
                    sleep(Duration::from_millis(300)).await; // 减少延迟时间以提升用户体验
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

#[derive(Debug)]
enum OpenerError {
    InvalidUrl(url::ParseError),
    InvalidScheme,
    InvalidPath(String),
    OpenError(String),
    CallbackError(String),
}

/// 验证路径是否有效
fn validate_path(path: &str) -> Result<(), OpenerError> {
    if path.is_empty() {
        return Err(OpenerError::InvalidPath("Path is empty".to_string()));
    }
    if !std::path::Path::new(path).exists() {
        return Err(OpenerError::InvalidPath(format!("Path does not exist: {}", path)));
    }
    Ok(())
}

/// 发送回调请求
async fn send_callback(callback_url: &str) -> Result<(), OpenerError> {
    reqwest::get(callback_url)
        .await
        .map_err(|e| OpenerError::CallbackError(e.to_string()))?;
    Ok(())
}

/// 处理 opener:// URL
fn handle_opener_url(url: &str) {
    let result = (|| -> Result<(), OpenerError> {
        let parsed_url = Url::parse(url).map_err(|e| {
            log::error!("URL parse error: {}", e);
            OpenerError::InvalidUrl(e)
        })?;
        
        if parsed_url.scheme() != "opener" {
            return Err(OpenerError::InvalidScheme);
        }
        
        log::info!("Processing deep link: {}", parsed_url);
        let app_handle = core::handle::Handle::global().app_handle().unwrap();

        // 获取回调 URL（支持多个参数名）
        let callback_url = parsed_url.query_pairs()
            .find(|(key, _)| key == "callback" || key == "after" || key == "do")
            .map(|(_, value)| value.to_string());

        if let Some(query_pairs) = parsed_url.query_pairs().find(|(key, _)| key == "path") {
            let path = query_pairs.1.to_string();
            validate_path(&path)?;
            
            log::info!("Open ready: {}", path);
            app_handle.opener().open_path(&path, None::<&str>)
                .map_err(|e| {
                    let err_msg = e.to_string();
                    log::error!("Failed to open path: {}", err_msg);
                    OpenerError::OpenError(err_msg)
                })?;
            
            log::info!("Opening path: {}", path);

            // 如果存在回调 URL，发送回调请求
            if let Some(callback) = callback_url {
                log::info!("Sending callback to: {}", callback);
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = send_callback(&callback).await {
                        match &e {
                            OpenerError::CallbackError(err_msg) => {
                                log::error!("Failed to send callback: {:?}, {}", callback, err_msg);
                            },
                            _ => log::error!("Unexpected error in callback: {:?}", e),
                        }
                    }
                });
            }

            Ok(())
        } else {
            Err(OpenerError::InvalidPath("No path parameter found".to_string()))
        }
    })();

    if let Err(e) = result {
        match &e {
            OpenerError::InvalidUrl(err) => log::error!("Failed to handle opener URL - Invalid URL: {}", err),
            OpenerError::InvalidScheme => log::error!("Failed to handle opener URL: Invalid scheme"),
            OpenerError::InvalidPath(path_err) => log::error!("Failed to handle opener URL - Path error: {}", path_err),
            OpenerError::OpenError(open_err) => log::error!("Failed to handle opener URL - Open error: {}", open_err),
            OpenerError::CallbackError(cb_err) => log::error!("Failed to handle opener URL - Callback error: {}", cb_err),
        }
    }
}
