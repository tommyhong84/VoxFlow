use futures_util::StreamExt;
use serde_json::json;
use tauri::Emitter;

use crate::core::agent::{AgentPlan, SuggestedCharacter};
use crate::core::cancel_token::CancellationToken;
use crate::core::error::AppError;
use crate::core::models::Character;

use super::parser::parse_agent_plan;

/// Analyze outline and return a structured plan with chapters,
/// suggested characters, and style — WITHOUT generating script lines yet.
/// This is Phase 1 of the two-phase Agent workflow.
/// Streams tokens back to the frontend for real-time feedback.
#[tauri::command]
pub async fn analyze_outline(
    app: tauri::AppHandle,
    cancel_token: tauri::State<'_, CancellationToken>,
    outline: String,
    api_endpoint: String,
    api_key: String,
    model: String,
    characters: Vec<Character>,
    enable_thinking: bool,
) -> Result<AgentPlan, AppError> {
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

    cancel_token.reset();

    // Emit tool-call event so UI shows this as a Skill invocation
    let _ = app.emit(
        "agent-tool-call",
        &json!({
            "tool": "outline_analysis",
            "query": outline.chars().take(100).collect::<String>()
        }),
    );

    let url = format!("{}/chat/completions", api_endpoint.trim_end_matches('/'));

    let existing_char_names: Vec<&str> = characters.iter().map(|c| c.name.as_str()).collect();

    let existing_chars_section = if existing_char_names.is_empty() {
        String::new()
    } else {
        format!("\nExisting project characters: {}\nTry to match suggested characters to these existing ones when possible.\n", existing_char_names.join(", "))
    };

    let system_prompt = format!(
        "You are an audiobook script planning assistant. Analyze the user's outline and return a structured plan.\n\n\
        CRITICAL LANGUAGE RULE: You MUST detect the language of the user's outline and respond entirely in that SAME language. \
        If the outline is in English, ALL content (titles, descriptions, notes) must be in English. \
        If the outline is in Chinese, respond in Chinese. Match the user's language exactly.\n\n\
        {existing_chars}\
        Requirements:\n\
        1. Identify chapters/scenes, estimate line count per chapter (be generous — aim for 15-30+ lines per chapter for rich dialogue), list involved characters, describe mood\n\
        2. Extract all characters with their roles (protagonist, antagonist, narrator, etc.)\n\
        3. Check if characters match existing project characters\n\
        4. Summarize overall style\n\
        5. Provide character configuration notes\n\n\
        Return ONLY valid JSON (no markdown fences):\n\
        {{\"chapters\":[{{\"title\":\"...\",\"estimated_lines\":20,\"characters\":[\"...\"],\"mood\":\"...\"}}],\
        \"suggested_characters\":[{{\"name\":\"...\",\"role\":\"...\",\"matched_existing\":false,\"existing_id\":null}}],\
        \"overall_style\":\"...\",\"character_notes\":\"...\"}}",
        existing_chars = existing_chars_section
    );

    let body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": outline }
        ],
        "stream": true,
        "max_tokens": 8192,
        "enable_thinking": enable_thinking
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| {
            let msg = format!("LLM request failed: {}", e);
            let _ = app.emit("llm-error", &msg);
            AppError::LlmService(msg)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        let msg = format!("LLM API error {}: {}", status, body_text);
        let _ = app.emit("llm-error", &msg);
        return Err(AppError::LlmService(msg));
    }

    // Read SSE stream chunk by chunk for real-time streaming
    let mut accumulated_text = String::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        // Check cancellation
        if cancel_token.is_cancelled() {
            let _ = app.emit("llm-complete", &());
            let _ = app.emit("llm-cancel", &());
            return Err(AppError::LlmService("Cancelled".to_string()));
        }

        let chunk = chunk_result.map_err(|e| {
            let msg = format!("Failed to read LLM response: {}", e);
            let _ = app.emit("llm-error", &msg);
            AppError::LlmService(msg)
        })?;

        let body_str = String::from_utf8_lossy(&chunk);
        for line in body_str.lines() {
            let line = line.trim();
            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                    // Emit thinking/reasoning content
                    if let Some(reasoning) =
                        parsed["choices"][0]["delta"]["reasoning_content"].as_str()
                    {
                        let _ = app.emit("llm-thinking", reasoning);
                    }
                    // Emit normal content
                    if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                        accumulated_text.push_str(content);
                        let _ = app.emit("llm-token", content);
                    }
                }
            }
        }
    }

    // Signal stream completion
    let _ = app.emit("llm-complete", &());

    let plan: AgentPlan = parse_agent_plan(&accumulated_text).map_err(|e| {
        let msg = format!(
            "Failed to parse plan: {}\nRaw: {}",
            e,
            accumulated_text.chars().take(300).collect::<String>()
        );
        let _ = app.emit("llm-error", &msg);
        AppError::LlmService(msg)
    })?;

    // Enrich: match suggested characters with existing project characters
    let char_map: std::collections::HashMap<String, &Character> =
        characters.iter().map(|c| (c.name.clone(), c)).collect();

    let enriched_chars: Vec<SuggestedCharacter> = plan
        .suggested_characters
        .into_iter()
        .map(|mut sc| {
            if let Some(existing) = char_map.get(&sc.name) {
                sc.matched_existing = true;
                sc.existing_id = Some(existing.id.clone());
            }
            sc
        })
        .collect();

    // Emit tool-result event
    let _ = app.emit(
        "agent-tool-result",
        &json!({
            "tool": "outline_analysis",
            "results_count": enriched_chars.len()
        }),
    );

    Ok(AgentPlan {
        chapters: plan.chapters,
        suggested_characters: enriched_chars,
        overall_style: plan.overall_style,
        character_notes: plan.character_notes,
    })
}
