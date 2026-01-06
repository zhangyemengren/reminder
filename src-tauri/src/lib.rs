mod notification;
mod timer;
mod tray;
mod window;

use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            notification::send_notification,
            timer::start_time_task
        ])
        .manage(timer::TimeState::new())
        .setup(|app| {
            // 1. 初始化托盘
            tray::create_tray(app)?;

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
