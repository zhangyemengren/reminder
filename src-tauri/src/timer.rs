use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use tokio::time::{self, Duration};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeStatus {
    Paused = 0,
    Running = 1,
    Finished = 2,
}

impl From<u8> for TimeStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => TimeStatus::Paused,
            1 => TimeStatus::Running,
            _ => TimeStatus::Finished,
        }
    }
}

#[derive(Default, serde::Serialize)]
pub struct TimeData {
    key: String,
    seconds: AtomicU32,
    status: AtomicU8,
}

impl TimeData {
    pub const EVENT_NAME: &'static str = "time-tick";

    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ..Default::default()
        }
    }

    pub fn get_seconds(&self) -> u32 {
        self.seconds.load(Ordering::SeqCst)
    }

    pub fn set_seconds(&self, seconds: u32) {
        self.seconds.store(seconds, Ordering::SeqCst);
    }

    pub fn get_status(&self) -> TimeStatus {
        self.status.load(Ordering::SeqCst).into()
    }

    pub fn set_status(&self, status: TimeStatus) {
        self.status.store(status as u8, Ordering::SeqCst);
    }
}

pub struct TimeManager {
    pub id: AtomicUsize,
    pub state: Mutex<HashMap<String, Arc<TimeData>>>,
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            id: AtomicUsize::new(0),
            state: Mutex::new(HashMap::new()),
        }
    }

    pub fn remove(&self, key: &str) {
        self.state.lock().unwrap().remove(key);
    }

    pub fn get(&self, key: &str) -> Option<Arc<TimeData>> {
        self.state.lock().unwrap().get(key).cloned()
    }
}

#[tauri::command]
pub async fn start_time_task(
    app: tauri::AppHandle,
    manager: tauri::State<'_, TimeManager>,
    seconds: u32,
) -> Result<String, String> {
    let key = format!("task_{}", manager.id.fetch_add(1, Ordering::SeqCst));
    let state = Arc::new(TimeData::new(&key));
    state.set_seconds(seconds);
    state.set_status(TimeStatus::Running);

    let state_clone = Arc::clone(&state);
    let key_clone = key.clone();
    manager.state.lock().unwrap().insert(key.clone(), state);

    tauri::async_runtime::spawn(async move {
        loop {
            let status = state_clone.get_status();

            // 已完成，清理并退出循环
            if status == TimeStatus::Finished {
                if let Some(manager) = app.try_state::<TimeManager>() {
                    manager.remove(&key_clone);
                }
                break;
            }

            // 暂停状态，等待后继续检查
            if status == TimeStatus::Paused {
                time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            // Running 状态
            let current_seconds = state_clone.get_seconds();

            if current_seconds == 0 {
                // 倒计时结束
                state_clone.set_status(TimeStatus::Finished);
                app.emit(TimeData::EVENT_NAME, &*state_clone).unwrap();
                println!("Time's up! Task: {}", key_clone);

                // 清理内存
                if let Some(manager) = app.try_state::<TimeManager>() {
                    manager.remove(&key_clone);
                }
                break;
            }

            // 发送当前状态
            app.emit(TimeData::EVENT_NAME, &*state_clone).unwrap();

            // 减少秒数
            state_clone.seconds.fetch_sub(1, Ordering::SeqCst);

            // 等待 1 秒
            time::sleep(Duration::from_secs(1)).await;
        }
    });

    Ok(key)
}

#[tauri::command]
pub fn pause_time_task(manager: tauri::State<'_, TimeManager>, key: String) -> Result<(), String> {
    if let Some(state) = manager.get(&key) {
        let current_status = state.get_status();
        if current_status == TimeStatus::Running {
            state.set_status(TimeStatus::Paused);
            Ok(())
        } else {
            Err(format!("Task {} is not running", key))
        }
    } else {
        Err(format!("Task {} not found", key))
    }
}

#[tauri::command]
pub fn resume_time_task(manager: tauri::State<'_, TimeManager>, key: String) -> Result<(), String> {
    if let Some(state) = manager.get(&key) {
        let current_status = state.get_status();
        if current_status == TimeStatus::Paused {
            state.set_status(TimeStatus::Running);
            Ok(())
        } else {
            Err(format!("Task {} is not paused", key))
        }
    } else {
        Err(format!("Task {} not found", key))
    }
}

#[tauri::command]
pub fn stop_time_task(manager: tauri::State<'_, TimeManager>, key: String) -> Result<(), String> {
    if let Some(state) = manager.get(&key) {
        state.set_status(TimeStatus::Finished);
        manager.remove(&key);
        Ok(())
    } else {
        Err(format!("Task {} not found", key))
    }
}
