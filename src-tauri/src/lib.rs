mod notification;
mod store;
mod timer;
mod tray;
mod window;

use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            notification::send_notification,
            timer::start_time_task,
            timer::get_event_key,
            store::get_store_value,
            store::set_store_value,
        ])
        .manage(timer::TimeState::new())
        .setup(|app| {
            // 1. 初始化托盘
            tray::create(app)?;
            // 2. 初始化持久数据
            store::AppStore::create(app)?;
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
