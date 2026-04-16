use std::sync::Mutex;

use crate::core::agent::{AgentConfig, AgentPlan};
use crate::core::cancel_token::CancellationToken;
use crate::core::db::Database;
use crate::core::error::AppError;
use crate::core::models::Character;

use super::utils::TauriEmitter;

/// Run the full agent pipeline: the agent autonomously decides the workflow
/// (analyze outline → extract characters → generate script → save).
/// This replaces the manual two-phase flow with an agent-driven approach.
#[tauri::command]
pub async fn run_agent_pipeline(
    app: tauri::AppHandle,
    db: tauri::State<'_, Mutex<Database>>,
    cancel_token: tauri::State<'_, CancellationToken>,
    project_id: String,
    outline: String,
    api_endpoint: String,
    api_key: String,
    model: String,
    characters: Vec<Character>,
    agent_plan: Option<AgentPlan>,
    extra_instructions: Option<String>,
    enable_thinking: bool,
) -> Result<(), AppError> {
    let config = AgentConfig {
        api_endpoint,
        api_key,
        model,
        enable_thinking,
        project_id: project_id.clone(),
    };

    let emitter = TauriEmitter(app);

    #[allow(deprecated)]
    crate::core::agent::run_agent_pipeline(
        &emitter,
        &cancel_token,
        &db,
        &config,
        &outline,
        characters,
        agent_plan.as_ref(),
        extra_instructions.as_deref(),
    )
    .await
}

/// Phase 1: Analyze outline and return a plan for user review.
/// Does NOT generate script — stops after analysis for user confirmation.
#[tauri::command]
pub async fn run_analysis_step(
    app: tauri::AppHandle,
    cancel_token: tauri::State<'_, CancellationToken>,
    outline: String,
    api_endpoint: String,
    api_key: String,
    model: String,
    characters: Vec<Character>,
    enable_thinking: bool,
) -> Result<AgentPlan, AppError> {
    let config = AgentConfig {
        api_endpoint,
        api_key,
        model,
        enable_thinking,
        project_id: String::new(), // Not used in analysis step
    };

    let emitter = TauriEmitter(app);

    crate::core::agent::run_analysis_step(&emitter, &cancel_token, &config, &outline, &characters)
        .await
}

/// Phase 2: Generate script from a confirmed plan.
/// Takes an optional plan (user may have modified it) and extra instructions.
#[tauri::command]
pub async fn run_generation_step(
    app: tauri::AppHandle,
    db: tauri::State<'_, Mutex<Database>>,
    cancel_token: tauri::State<'_, CancellationToken>,
    project_id: String,
    outline: String,
    api_endpoint: String,
    api_key: String,
    model: String,
    characters: Vec<Character>,
    plan: Option<AgentPlan>,
    extra_instructions: Option<String>,
    enable_thinking: bool,
) -> Result<(), AppError> {
    let config = AgentConfig {
        api_endpoint,
        api_key,
        model,
        enable_thinking,
        project_id: project_id.clone(),
    };

    let emitter = TauriEmitter(app);

    crate::core::agent::run_generation_step(
        &emitter,
        &cancel_token,
        &db,
        &config,
        &outline,
        &characters,
        plan.as_ref(),
        extra_instructions.as_deref(),
    )
    .await
}

/// Revise specific sections of an existing script with new instructions.
#[tauri::command]
pub async fn run_revision_step(
    app: tauri::AppHandle,
    db: tauri::State<'_, Mutex<Database>>,
    cancel_token: tauri::State<'_, CancellationToken>,
    project_id: String,
    outline: String,
    api_endpoint: String,
    api_key: String,
    model: String,
    characters: Vec<Character>,
    instructions: String,
    section_indices: Option<Vec<usize>>,
    plan: Option<AgentPlan>,
    enable_thinking: bool,
) -> Result<(), AppError> {
    let revision = crate::core::agent::RevisionRequest {
        instructions,
        section_indices,
    };

    let config = AgentConfig {
        api_endpoint,
        api_key,
        model,
        enable_thinking,
        project_id: project_id.clone(),
    };

    let emitter = TauriEmitter(app);

    crate::core::agent::run_revision_step(
        &emitter,
        &cancel_token,
        &db,
        &config,
        &outline,
        &characters,
        &revision,
        plan.as_ref(),
    )
    .await
}
