use crate::core::models::Character;
use tauri::Emitter;

/// Resolve a character name to its ID.
pub(crate) fn resolve_character(name: &Option<String>, characters: &[Character]) -> Option<String> {
    name.as_ref().and_then(|n| {
        characters
            .iter()
            .find(|c| c.name == *n)
            .map(|c| c.id.clone())
    })
}

/// Tauri-specific event emitter — bridges AppHandle::emit to EventEmitter trait.
pub(crate) struct TauriEmitter(pub tauri::AppHandle);

impl crate::core::event_emitter::EventEmitter for TauriEmitter {
    fn emit_json(&self, event: &str, payload: &serde_json::Value) {
        let _ = self.0.emit(event, payload);
    }
}
