use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::time::{self, Duration};

#[derive(Default)]
pub struct TimeState {
    seconds: Arc<AtomicU32>,
}
impl TimeState {
    pub const EVENT_NAME: &'static str = "time-tick";
    pub fn new() -> Self {
        Self::default()
    }
    pub const fn get_event_name() -> &'static str {
        Self::EVENT_NAME
    }
    pub fn get_seconds(&self) -> u32 {
        self.seconds.load(Ordering::SeqCst)
    }
    pub fn set_seconds(&self, seconds: u32) {
        self.seconds.store(seconds, Ordering::SeqCst);
    }
}

#[tauri::command]
pub async fn start_time_task(app: tauri::AppHandle, seconds: u32) {
    let state = app.state::<TimeState>();
    state.set_seconds(seconds);
    let seconds_left = Arc::clone(&state.seconds);

    tauri::async_runtime::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let previous_value = seconds_left.load(Ordering::SeqCst);
            if previous_value > 0 {
                app.emit(TimeState::EVENT_NAME, previous_value).unwrap();
                seconds_left.fetch_sub(1, Ordering::SeqCst);
            } else {
                app.emit(TimeState::EVENT_NAME, 0).unwrap();
                println!("Time's up!");
                break;
            }
        }
    });
}
#[tauri::command]
pub fn get_event_key(app: tauri::AppHandle) -> &'static str {
    TimeState::EVENT_NAME
}
