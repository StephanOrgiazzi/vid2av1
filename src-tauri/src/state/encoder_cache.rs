use tauri::{AppHandle, Manager};

use super::types::Av1EncoderCache;

pub fn get_cached_av1_encoders(app: &AppHandle) -> Option<Vec<String>> {
    app.state::<Av1EncoderCache>()
        .0
        .lock()
        .ok()
        .and_then(|cache| cache.clone())
}

pub fn set_cached_av1_encoders(app: &AppHandle, encoders: Vec<String>) {
    if let Ok(mut cache) = app.state::<Av1EncoderCache>().0.lock() {
        *cache = Some(encoders);
    }
}
