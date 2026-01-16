use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

pub(crate) async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;
        let main_window = app.get_webview_window("main").unwrap();
        main_window.set_title("正在更新...")?;
        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    main_window.set_title(&format!("正在更新...({downloaded} / {content_length:?})")).unwrap();
                },
                || {
                    main_window.set_title("更新完成, 即将重启...").unwrap();
                },
            )
            .await?;

        app.restart();
    }

    Ok(())
}
