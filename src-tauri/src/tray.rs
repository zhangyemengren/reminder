use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Runtime,
};

pub fn create_tray(app: &mut tauri::App) -> tauri::Result<TrayIcon> {
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_item])?;

    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(handle_menu_event) // 绑定菜单处理函数
        .build(app)?;

    Ok(tray)
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        "quit" => app.exit(0),
        _ => {}
    }
}
