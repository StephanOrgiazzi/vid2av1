use std::process::ChildStdin;
use std::sync::Arc;

use tauri::{AppHandle, Manager};

use super::types::{ActiveConversionControl, ActiveFfmpegPids, ConversionControl};

pub fn register_ffmpeg_pid(app: &AppHandle, pid: u32) -> Result<(), String> {
    let state = app.state::<ActiveFfmpegPids>();
    let mut pids = state
        .0
        .lock()
        .map_err(|_| "Failed to lock ffmpeg process state.".to_string())?;
    pids.insert(pid);
    Ok(())
}

pub fn unregister_ffmpeg_pid(app: &AppHandle, pid: u32) {
    if let Ok(mut pids) = app.state::<ActiveFfmpegPids>().0.lock() {
        pids.remove(&pid);
    }
}

pub fn register_active_conversion(
    app: &AppHandle,
    pid: u32,
    stdin: ChildStdin,
) -> Result<(), String> {
    let state = app.state::<ActiveConversionControl>();
    let mut active = state
        .0
        .lock()
        .map_err(|_| "Failed to lock active conversion state.".to_string())?;
    if active.is_some() {
        return Err("Another conversion is already running.".to_string());
    }
    *active = Some(Arc::new(ConversionControl::new(pid, stdin)));
    Ok(())
}

pub fn unregister_active_conversion(app: &AppHandle, pid: u32) {
    if let Ok(mut active) = app.state::<ActiveConversionControl>().0.lock() {
        let should_clear = active
            .as_ref()
            .map(|control| control.pid() == pid)
            .unwrap_or(false);
        if should_clear {
            *active = None;
        }
    }
}

pub(super) fn get_active_conversion(app: &AppHandle) -> Result<Arc<ConversionControl>, String> {
    let state = app.state::<ActiveConversionControl>();
    let active = state
        .0
        .lock()
        .map_err(|_| "Failed to lock active conversion state.".to_string())?;
    active
        .as_ref()
        .cloned()
        .ok_or_else(|| "No conversion is currently running.".to_string())
}

pub(super) fn set_active_conversion_cancel_requested(app: &AppHandle, requested: bool) {
    if let Ok(active) = app.state::<ActiveConversionControl>().0.lock() {
        if let Some(control) = active.as_ref() {
            if requested {
                control.mark_cancel_requested();
            } else {
                control.clear_cancel_requested();
            }
        }
    }
}

pub(super) fn is_active_conversion_cancel_requested(app: &AppHandle) -> bool {
    if let Ok(active) = app.state::<ActiveConversionControl>().0.lock() {
        return active
            .as_ref()
            .map(|control| control.is_cancel_requested())
            .unwrap_or(false);
    }
    false
}

pub(super) fn collect_registered_ffmpeg_pids(app: &AppHandle) -> Vec<u32> {
    if let Ok(pids) = app.state::<ActiveFfmpegPids>().0.lock() {
        return pids.iter().copied().collect();
    }
    Vec::new()
}

pub(super) fn clear_registered_ffmpeg_pids(app: &AppHandle) {
    if let Ok(mut pids) = app.state::<ActiveFfmpegPids>().0.lock() {
        pids.clear();
    }
}
