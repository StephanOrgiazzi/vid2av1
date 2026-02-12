use tauri::{AppHandle, Manager};

use super::types::Av1EncoderCache;

pub fn get_cached_av1_encoders(app: &AppHandle) -> Result<Option<Vec<String>>, String> {
    let state = app.state::<Av1EncoderCache>();
    let cache = state
        .0
        .lock()
        .map_err(|_| "Failed to lock AV1 encoder cache.".to_string())?;
    Ok(cache.clone())
}

pub fn set_cached_av1_encoders(app: &AppHandle, encoders: Vec<String>) -> Result<(), String> {
    let state = app.state::<Av1EncoderCache>();
    let mut cache = state
        .0
        .lock()
        .map_err(|_| "Failed to lock AV1 encoder cache.".to_string())?;
    *cache = Some(encoders);
    Ok(())
}
