use std::sync::Mutex;

use crate::core::config::ConfigManager;
use crate::core::db::Database;
use crate::core::error::AppError;
use crate::core::models::UserSettings;

#[tauri::command]
pub fn save_settings(
    db: tauri::State<'_, Mutex<Database>>,
    settings: UserSettings,
) -> Result<(), AppError> {
    let db = db.lock().map_err(|e| AppError::Database(e.to_string()))?;
    db.save_settings(&settings)
}

#[tauri::command]
pub fn load_settings(
    db: tauri::State<'_, Mutex<Database>>,
) -> Result<UserSettings, AppError> {
    let db = db.lock().map_err(|e| AppError::Database(e.to_string()))?;
    db.load_settings()
}

#[tauri::command]
pub fn save_api_key(
    app: tauri::AppHandle,
    service: String,
    key: String,
) -> Result<(), AppError> {
    let config = ConfigManager::new(app);
    config.save_api_key(&service, &key)
}

#[tauri::command]
pub fn load_api_key(
    app: tauri::AppHandle,
    service: String,
) -> Result<Option<String>, AppError> {
    let config = ConfigManager::new(app);
    config.load_api_key(&service)
}
