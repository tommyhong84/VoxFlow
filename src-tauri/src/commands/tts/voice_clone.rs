use tauri::Manager;

use crate::core::error::AppError;
use crate::core::models::VoiceConfig;
use log::info;

use super::models::VoiceEnrollmentOutput;
use super::websocket::{ws_realtime_connect, ws_realtime_run_task};

/// Create a cloned voice from uploaded/recorded audio via DashScope voice enrollment API.
#[tauri::command]
pub async fn create_voice(
    app: tauri::AppHandle,
    project_id: String,
    audio_data_base64: String,
    preferred_name: String,
    target_model: String,
) -> Result<String, AppError> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    info!(
        "[VoiceClone] create_voice: project={}, name={}, model={}",
        project_id, preferred_name, target_model
    );

    let api_key = {
        let config = crate::core::config::ConfigManager::new(app.clone());
        config
            .load_api_key("dashscope")
            .map_err(|e| AppError::Config(format!("Failed to load API key: {}", e)))?
            .ok_or_else(|| AppError::Config("DashScope API key not configured".into()))?
    };

    // Decode base64 audio
    let audio_bytes = STANDARD
        .decode(&audio_data_base64)
        .map_err(|e| AppError::FileSystem(format!("base64 decode: {}", e)))?;

    // Re-encode as data URI (the audio can be any common format: mp3, wav, webm)
    let data_uri = format!("data:audio/mpeg;base64,{}" , STANDARD.encode(&audio_bytes));

    let url = "https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization";
    let payload = serde_json::json!({
        "model": "qwen-voice-enrollment",
        "input": {
            "action": "create",
            "target_model": target_model,
            "preferred_name": preferred_name,
            "audio": { "data": data_uri }
        }
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::TtsService(format!("HTTP request failed: {}", e)))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| AppError::TtsService(format!("Read response body failed: {}", e)))?;

    if !status.is_success() {
        return Err(AppError::TtsService(format!(
            "Voice enrollment failed ({}): {}",
            status, body
        )));
    }

    let output: VoiceEnrollmentOutput = serde_json::from_str(&body)
        .map_err(|e| AppError::TtsService(format!("Parse response failed: {}, body: {}", e, body)))?;

    info!("[VoiceClone] Voice created: {}", output.output.voice);
    Ok(output.output.voice)
}

/// Preview a cloned voice by synthesizing a short test sentence via WS realtime.
/// Returns the path to the generated preview audio file.
#[tauri::command]
pub async fn preview_voice(
    app: tauri::AppHandle,
    project_id: String,
    voice: String,
    target_model: String,
) -> Result<String, AppError> {
    info!(
        "[VoiceClone] preview_voice: project={}, voice={}, model={}",
        project_id, voice, target_model
    );

    let api_key = {
        let config = crate::core::config::ConfigManager::new(app.clone());
        config
            .load_api_key("dashscope")
            .map_err(|e| AppError::Config(format!("Failed to load API key: {}", e)))?
            .ok_or_else(|| AppError::Config("DashScope API key not configured".into()))?
    };

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::FileSystem(e.to_string()))?;

    let preview_dir = app_data_dir
        .join("projects")
        .join(&project_id)
        .join("previews");
    std::fs::create_dir_all(&preview_dir)
        .map_err(|e| AppError::FileSystem(format!("mkdir previews: {}", e)))?;

    let preview_id = uuid::Uuid::new_v4().to_string();
    let file_path = preview_dir
        .join(format!("{}.mp3", &preview_id))
        .to_string_lossy()
        .to_string();

    // Use WS realtime to synthesize a short preview sentence
    let voice_config = VoiceConfig {
        voice_name: voice,
        tts_model: target_model.clone(),
        speed: 1.0,
        pitch: 1.0,
    };

    let preview_text = "你好，这是我的专属声音";
    let audio_bytes = {
        let ws = ws_realtime_connect(&api_key, &target_model).await?;
        let mut ws = ws;
        ws_realtime_run_task(
            &mut ws,
            &[preview_text],
            &voice_config,
            None,
            &target_model,
        )
        .await?
    };

    std::fs::write(&file_path, &audio_bytes)
        .map_err(|e| AppError::FileSystem(format!("write preview: {}", e)))?;

    info!("[VoiceClone] Preview audio saved: {}", file_path);
    Ok(file_path)
}
