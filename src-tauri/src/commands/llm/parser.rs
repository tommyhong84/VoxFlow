use crate::core::agent::AgentPlan;
use crate::core::models::{LlmScriptLine, LlmScriptResponse, LlmSection};

/// Extract the first JSON object from text that may contain surrounding prose.
/// Finds the first `{` and the matching last `}` to extract the JSON body.
pub(crate) fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end > start {
        Some(&text[start..=end])
    } else {
        None
    }
}

/// Parse LLM response into AgentPlan.
/// Handles markdown fences, surrounding prose text, and thinking-mode artifacts.
pub(crate) fn parse_agent_plan(text: &str) -> Result<AgentPlan, String> {
    let trimmed = text.trim();

    // Strip markdown code block: ```json ... ``` or ``` ... ```
    let stripped = if trimmed.starts_with("```") {
        if let Some(first_newline) = trimmed.find('\n') {
            let after_fence = &trimmed[first_newline + 1..];
            after_fence
                .trim()
                .strip_suffix("```")
                .unwrap_or(after_fence.trim())
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    // Try direct parse first
    if let Ok(plan) = serde_json::from_str::<AgentPlan>(stripped) {
        return Ok(plan);
    }

    // Extract JSON object from surrounding prose (common in thinking mode)
    if let Some(json_str) = extract_json_object(stripped) {
        if let Ok(plan) = serde_json::from_str::<AgentPlan>(json_str) {
            return Ok(plan);
        }
        // Try auto-completing truncated JSON
        let completed = auto_complete_json(json_str);
        if let Ok(plan) = serde_json::from_str::<AgentPlan>(&completed) {
            return Ok(plan);
        }
    }

    Err(format!(
        "Cannot parse AgentPlan from: {}",
        stripped.chars().take(300).collect::<String>()
    ))
}

/// Auto-complete truncated JSON by appending missing closing delimiters.
pub(crate) fn auto_complete_json(json: &str) -> String {
    let mut result = json.to_string();
    let mut in_string = false;
    let mut escape_next = false;
    let mut bracket_depth: usize = 0;
    let mut array_depth: usize = 0;

    for ch in result.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string {
            match ch {
                '{' => bracket_depth += 1,
                '}' => bracket_depth = bracket_depth.saturating_sub(1),
                '[' => array_depth += 1,
                ']' => array_depth = array_depth.saturating_sub(1),
                _ => {}
            }
        }
    }

    // Close any open string
    if in_string {
        result.push('"');
    }
    // Close arrays
    for _ in 0..array_depth {
        result.push(']');
    }
    // Close objects
    for _ in 0..bracket_depth {
        result.push('}');
    }

    result
}

/// Parse LLM response text as JSON LlmScriptResponse.
/// Strips markdown code block fences if present.
/// Backward compatible: accepts both new `{"sections":[...]}` format
/// and old `{"lines":[...]}` format (wraps lines in a default "正文" section).
/// Handles truncated JSON from stream cutoff.
pub(crate) fn parse_llm_json(text: &str) -> Result<LlmScriptResponse, String> {
    let trimmed = text.trim();

    // Strip markdown code block: ```json ... ``` or ``` ... ```
    let stripped = if trimmed.starts_with("```") {
        if let Some(first_newline) = trimmed.find('\n') {
            let after_fence = &trimmed[first_newline + 1..];
            after_fence
                .trim()
                .strip_suffix("```")
                .unwrap_or(after_fence.trim())
        } else {
            trimmed
                .trim()
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
                .unwrap_or(trimmed)
        }
    } else {
        trimmed
    };

    // Extract JSON object from surrounding prose (common in thinking mode)
    let json_str = extract_json_object(stripped).unwrap_or(stripped);

    // Try new sections format first
    if let Ok(resp) = serde_json::from_str::<LlmScriptResponse>(json_str) {
        return Ok(resp);
    }

    // Try auto-completing truncated JSON
    let completed = auto_complete_json(json_str);
    if let Ok(resp) = serde_json::from_str::<LlmScriptResponse>(&completed) {
        return Ok(resp);
    }

    // Fallback: try old lines format and wrap in default section
    let try_old_format = |s: &str| -> Option<LlmScriptResponse> {
        let value = serde_json::from_str::<serde_json::Value>(s).ok()?;
        let lines_array = value.get("lines")?.as_array()?;
        let lines: Vec<LlmScriptLine> = lines_array
            .iter()
            .filter_map(|l| serde_json::from_value::<LlmScriptLine>(l.clone()).ok())
            .collect();
        if lines.is_empty() {
            return None;
        }
        Some(LlmScriptResponse {
            sections: vec![LlmSection {
                title: "Main".to_string(),
                lines,
            }],
        })
    };

    if let Some(resp) = try_old_format(json_str) {
        return Ok(resp);
    }
    if let Some(resp) = try_old_format(&completed) {
        return Ok(resp);
    }

    Err(format!(
        "Cannot parse as sections or lines format, raw: {}",
        json_str.chars().take(500).collect::<String>()
    ))
}
