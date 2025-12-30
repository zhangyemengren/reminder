// src/window.rs
use tauri::{Runtime, Window, WindowEvent};

/// 窗口事件的统一分发器
pub fn handle_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            // 拦截关闭请求，改为隐藏窗口
            api.prevent_close();
            window.hide().unwrap();
        }
        WindowEvent::Focused(focused) => {
            if *focused {
                println!("窗口 {} 获得了焦点", window.label());
            }
        }
        // 你可以继续在这里添加更多事件处理
        _ => {}
    }
}
