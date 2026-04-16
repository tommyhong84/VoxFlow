//! LLM (Large Language Model) module
//!
//! This module provides LLM-based functionality for:
//! - Outline analysis and script planning
//! - Script generation from outlines
//! - Agent-driven pipeline workflow
//! - Knowledge base operations

mod commands;
mod generation;
mod kb;
mod outline;
mod parser;
mod pipeline;
mod utils;

// Re-export all public items from submodules (includes __cmd__ functions from #[tauri::command])
pub use commands::*;
pub use generation::*;
pub use kb::*;
pub use outline::*;
pub use pipeline::*;
