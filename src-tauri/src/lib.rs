mod notification;
mod store;
mod timer;
mod tray;
mod window;
mod update;

use tauri::{Manager, RunEvent};
use update::update;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            notification::send_notification,
            timer::start_time_task,
            timer::pause_time_task,
            timer::resume_time_task,
            timer::stop_time_task,
            store::get_store_value,
            store::set_store_value,
        ])
        .manage(timer::TimeManager::new())
        .setup(|app| {
            // 1. 初始化托盘
            tray::create(app)?;
            // 2. 初始化持久数据
            store::AppStore::create(app)?;
            let main_window = app.get_webview_window("main").unwrap();
            let welcome_window = app.get_webview_window("welcome").unwrap();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                welcome_window.hide().unwrap();
                main_window.show().unwrap();
                update(app_handle).await.unwrap();
            });
            Ok(())
        })
        .on_window_event(window::handle_event)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            handle_app_lifecycle(app_handle, event);
        });
}

fn handle_app_lifecycle(app_handle: &tauri::AppHandle, event: tauri::RunEvent) {
    match event {
        #[cfg(target_os = "macos")]
        RunEvent::Reopen {
            has_visible_windows,
            ..
        } => {
            if !has_visible_windows {
                if let Some(window) = app_handle.get_webview_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
        }
        _ => {}
    }
}
