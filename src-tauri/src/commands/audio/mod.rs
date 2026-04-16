//! Audio module
//!
//! This module provides audio functionality for:
//! - Audio playback via rodio
//! - Audio export and mixing
//! - Audio import (BGM and recordings)
//! - FFmpeg integration

mod export;
mod ffmpeg;
mod import;
mod player;
mod utils;

// Re-export all public items from submodules (includes __cmd__ functions from #[tauri::command])
pub use export::*;
pub use ffmpeg::{build_ffmpeg_args, find_ffmpeg};
pub use import::*;
pub use player::*;
