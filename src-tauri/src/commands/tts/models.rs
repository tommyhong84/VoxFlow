/// Maximum characters per TTS chunk.
/// The API limit for qwen3-tts-instruct-flash is 600 (likely byte-counted).
/// Chinese chars are 3 bytes in UTF-8, so 600 bytes ≈ 200 Chinese chars.
/// Use 200 as a safe threshold.
pub const TTS_CHUNK_MAX_CHARS: usize = 200;

/// Chunk with metadata about the pause to insert after it (in ms).
pub(crate) struct TtsChunk {
    pub text: String,
    /// Pause duration in ms to insert AFTER this chunk, before the next one.
    /// 0 for the last chunk.
    pub pause_ms: u32,
}

/// Response from the voice enrollment API.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct VoiceEnrollmentOutput {
    pub output: VoiceEnrollmentResult,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct VoiceEnrollmentResult {
    pub voice: String,
}
