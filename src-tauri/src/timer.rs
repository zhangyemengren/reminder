use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::time::{self, Duration};

#[derive(Default)]
pub struct TimeState {
    seconds: Arc<AtomicU32>,
}
impl TimeState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_seconds(&self) -> u32 {
        self.seconds.load(Ordering::SeqCst)
    }
    pub fn set_seconds(&self, seconds: u32) {
        self.seconds.store(seconds, Ordering::SeqCst);
    }
}

#[tauri::command]
pub fn start_time_task(app: tauri::AppHandle, seconds: u32) {
    let state = app.state::<TimeState>();
    state.set_seconds(seconds);
    let seconds_left = Arc::clone(&state.seconds);

    tauri::async_runtime::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let previous_value = seconds_left.load(Ordering::SeqCst);
            if previous_value > 0 {
                seconds_left.fetch_sub(1, Ordering::SeqCst);
                app.emit("time-tick", previous_value).unwrap();
            } else {
                app.emit("time-tick", 0).unwrap();
                println!("Time's up!");
                break;
            }
        }
    });
}
