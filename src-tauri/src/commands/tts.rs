use std::sync::Mutex;

use tauri::Manager;

use crate::core::db::Database;
use crate::core::error::AppError;
use crate::core::models::{AudioFragment, VoiceConfig};
use log::{debug, info, error};

/// Build the audio file path for a given project and line.
/// Returns `{app_data_dir}/projects/{project_id}/audio/{line_id}.mp3`
pub fn build_audio_path(
    app_data_dir: &std::path::Path,
    project_id: &str,
    line_id: &str,
) -> std::path::PathBuf {
    app_data_dir
        .join("projects")
        .join(project_id)
        .join("audio")
        .join(format!("{}.mp3", line_id))
}

#[tauri::command]
pub async fn generate_tts(
    app: tauri::AppHandle,
    db: tauri::State<'_, Mutex<Database>>,
    project_id: String,
    line_id: String,
    text: String,
    voice_config: VoiceConfig,
    api_key: String,
) -> Result<AudioFragment, AppError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::FileSystem(e.to_string()))?;

    let audio_path = build_audio_path(&app_data_dir, &project_id, &line_id);

    if let Some(parent) = audio_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::FileSystem(format!("Failed to create audio directory: {}", e))
        })?;
    }

    let audio_bytes =
        call_dashscope_tts(&text, &voice_config, &api_key).await?;

    std::fs::write(&audio_path, &audio_bytes).map_err(|e| {
        AppError::FileSystem(format!("Failed to write audio file: {}", e))
    })?;

    // Re-encode with FFmpeg to fix VBR headers and ensure valid MP3 format.
    // TTS services may return WAV disguised as .mp3 or VBR MP3 with bad headers.
    let tmp_path = audio_path.with_extension("tmp.mp3");
    let ffmpeg_result = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", &audio_path.to_string_lossy(),
            "-codec:a", "libmp3lame",
            "-b:a", "192k",
            &tmp_path.to_string_lossy(),
        ])
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .output();

    match ffmpeg_result {
        Ok(output) if output.status.success() => {
            // Replace original with re-encoded file
            std::fs::rename(&tmp_path, &audio_path).map_err(|e| {
                AppError::FileSystem(format!("Failed to replace audio file: {}", e))
            })?;
        }
        _ => {
            // FFmpeg failed or not found — keep original file, clean up tmp
            let _ = std::fs::remove_file(&tmp_path);
        }
    }

    let file_path = audio_path.to_string_lossy().to_string();
    let fragment = AudioFragment {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: project_id.clone(),
        line_id: line_id.clone(),
        file_path,
        duration_ms: None,
    };

    let db = db.lock().map_err(|e| AppError::Database(e.to_string()))?;
    db.upsert_audio_fragment(&fragment)?;

    Ok(fragment)
}

/// 调用阿里百炼 (DashScope) Qwen-TTS / CosyVoice 服务生成音频。
///
/// API 端点（北京地域）：
/// POST https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation
///
/// voice_config.tts_model 指定模型名（如 "qwen3-tts-flash"、"cosyvoice-v3-flash"）
/// voice_config.voice_name 指定音色（如 "Cherry"、"longanyang"）
async fn call_dashscope_tts(
    text: &str,
    voice_config: &VoiceConfig,
    api_key: &str,
) -> Result<Vec<u8>, AppError> {
    use reqwest::header::CONTENT_TYPE;
    use serde_json::json;

    let model = if voice_config.tts_model.is_empty() {
        "qwen3-tts-flash"
    } else {
        &voice_config.tts_model
    };

    let body = json!({
        "model": model,
        "input": {
            "text": text, 
            "voice": voice_config.voice_name,
            // "language_type": "Chinese",
        }
    });

    // 使用 debug! 宏，只有在设置 RUST_LOG=debug 时才会显示，避免生产环境日志过多
    debug!("🚀 正在构造 TTS 请求:");
    // serde_json::to_string_pretty 可以格式化输出 JSON，方便阅读
    debug!("{}", serde_json::to_string_pretty(&body).unwrap()); 
    

    let client = reqwest::Client::new();
    let response = client
        .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation")
        .header(CONTENT_TYPE, "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| AppError::TtsService(format!("百炼 TTS 请求失败: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        return Err(AppError::TtsService(format!(
            "百炼 TTS API 错误 {}: {}",
            status, body_text
        )));
    }

    let resp_body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::TtsService(format!("解析百炼 TTS 响应失败: {}", e)))?;

    // Non-streaming: response contains audio URL in output.audio.url
    if let Some(url) = resp_body["output"]["audio"]["url"].as_str() {
        let audio_response = client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::TtsService(format!("下载百炼 TTS 音频失败: {}", e)))?;

        return audio_response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| AppError::TtsService(format!("读取百炼 TTS 音频失败: {}", e)));
    }

    // Fallback: base64-encoded audio data
    if let Some(data) = resp_body["output"]["audio"]["data"].as_str() {
        use base64::Engine;
        return base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| AppError::TtsService(format!("解码百炼 TTS 音频失败: {}", e)));
    }

    Err(AppError::TtsService(format!(
        "百炼 TTS 响应中未找到音频数据: {}",
        resp_body
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_audio_path() {
        let app_data = std::path::Path::new("/data");
        let path = build_audio_path(app_data, "proj-1", "line-42");
        assert_eq!(
            path,
            std::path::PathBuf::from("/data/projects/proj-1/audio/line-42.mp3")
        );
    }
}
