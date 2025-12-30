use tauri_plugin_notification::NotificationExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub fn send_notification(app: tauri::AppHandle, title: &str, body: &str) {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .unwrap();
}


#[tauri::command]
pub fn count_down(app: tauri::AppHandle) {
    // TODO
}
