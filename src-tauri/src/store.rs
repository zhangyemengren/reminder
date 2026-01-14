use std::sync::Arc;
use tauri::{Manager, Runtime};
use tauri_plugin_store::{Store, StoreExt};

pub struct AppStore;

impl AppStore {
    pub const FILE_NAME: &'static str = "store.json";
    pub fn create<R: Runtime, M: Manager<R>>(
        manager: &M,
    ) -> tauri_plugin_store::Result<Arc<Store<R>>> {
        manager.store(Self::FILE_NAME)
    }
    pub fn get_value<R: Runtime, M: Manager<R>>(
        manager: &M,
        key: String,
    ) -> tauri_plugin_store::Result<Option<serde_json::Value>> {
        let store = manager.store(Self::FILE_NAME)?;
        Ok(store.get(&key))
    }
    pub fn set_value<R: Runtime, M: Manager<R>>(
        manager: &M,
        key: String,
        value: serde_json::Value,
    ) -> tauri_plugin_store::Result<()> {
        let store = manager.store(Self::FILE_NAME)?;
        store.set(key, value);
        Ok(())
    }
}

#[tauri::command]
pub async fn get_store_value(app: tauri::AppHandle, key: String) -> Option<serde_json::Value> {
    AppStore::get_value(&app, key).ok()?
}

#[tauri::command]
pub async fn set_store_value(app: tauri::AppHandle, key: String, value: serde_json::Value) {
    AppStore::set_value(&app, key, value).ok();
}
