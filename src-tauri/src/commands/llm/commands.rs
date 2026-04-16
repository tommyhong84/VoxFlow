use std::sync::Mutex;

use crate::core::cancel_token::CancellationToken;
use crate::core::db::Database;
use crate::core::error::AppError;
use crate::core::models::{ScriptLine, ScriptSection};

/// Cancel an ongoing LLM request (analyze_outline or generate_script).
#[tauri::command]
pub fn cancel_llm(cancel_token: tauri::State<'_, CancellationToken>) {
    cancel_token.cancel();
}

#[tauri::command]
pub fn save_script(
    db: tauri::State<'_, Mutex<Database>>,
    project_id: String,
    lines: Vec<ScriptLine>,
    sections: Vec<ScriptSection>,
) -> Result<(), AppError> {
    let db = db.lock().map_err(|e| AppError::Database(e.to_string()))?;
    db.save_script(&project_id, &lines, &sections)
}

#[tauri::command]
pub fn load_script(
    db: tauri::State<'_, Mutex<Database>>,
    project_id: String,
) -> Result<Vec<ScriptLine>, AppError> {
    let db = db.lock().map_err(|e| AppError::Database(e.to_string()))?;
    db.load_script(&project_id)
}
