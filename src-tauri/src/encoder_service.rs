use tauri::AppHandle;

use crate::ffmpeg::{list_encoders, resolve_encoder_candidates, resolve_tool_path};
use crate::state::{get_cached_av1_encoders, set_cached_av1_encoders};

pub fn get_available_av1_encoders(app: &AppHandle) -> Result<Vec<String>, String> {
    if let Some(cached) = get_cached_av1_encoders(app)? {
        return Ok(cached);
    }

    let ffmpeg_path = resolve_tool_path(app, "ffmpeg.exe")?;
    let discovered = list_encoders(&ffmpeg_path)?;
    set_cached_av1_encoders(app, discovered.clone())?;
    Ok(discovered)
}

pub fn pick_auto_av1_encoder(app: &AppHandle) -> Result<String, String> {
    let available = get_available_av1_encoders(app)?;
    let candidates = resolve_encoder_candidates(None, &available)?;
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| "No AV1 encoder selected.".to_string())
}
