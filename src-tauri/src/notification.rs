use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub fn send_notification(app: tauri::AppHandle, title: &str, body: &str) {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .unwrap();
}
