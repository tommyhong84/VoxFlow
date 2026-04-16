//! TTS (Text-to-Speech) module
//!
//! This module provides text-to-speech functionality using various backends:
//! - WebSocket realtime API for Qwen TTS models
//! - HTTP REST API for non-realtime models
//! - Voice cloning capabilities

mod commands;
mod core;
mod http;
mod models;
mod utils;
mod voice_clone;
mod websocket;

// Re-export all public items from submodules (includes __cmd__ functions from #[tauri::command])
pub use commands::*;
pub use utils::get_audio_duration;
pub use voice_clone::*;
