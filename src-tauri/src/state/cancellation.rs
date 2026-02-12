use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager};

use crate::error_protocol::{is_error_code, ERROR_CODE_NO_ACTIVE_CONVERSION};

use super::process_registry::set_active_conversion_cancel_requested;
use super::process_registry::{get_active_conversion, is_active_conversion_cancel_requested};
use super::termination::terminate_all_active_ffmpeg;
use super::types::GlobalCancelFlag;

pub fn cancel_active_conversion(app: &AppHandle) -> Result<(), String> {
    request_cancel(app);

    match get_active_conversion(app) {
        Ok(control) => {
            control.mark_cancel_requested();
            let _ = control.send_quit_command();
        }
        Err(error) => {
            if !is_error_code(&error, ERROR_CODE_NO_ACTIVE_CONVERSION) {
                return Err(error);
            }
        }
    }

    terminate_all_active_ffmpeg(app)?;
    Ok(())
}

pub fn is_cancel_requested(app: &AppHandle) -> bool {
    if app.state::<GlobalCancelFlag>().0.load(Ordering::Relaxed) {
        return true;
    }

    match is_active_conversion_cancel_requested(app) {
        Ok(requested) => requested,
        Err(error) => {
            eprintln!("Failed to read active conversion cancel state: {error}");
            true
        }
    }
}

pub fn clear_cancel_requested(app: &AppHandle) -> Result<(), String> {
    app.state::<GlobalCancelFlag>()
        .0
        .store(false, Ordering::Relaxed);
    set_active_conversion_cancel_requested(app, false)
}

pub fn request_cancel(app: &AppHandle) {
    app.state::<GlobalCancelFlag>()
        .0
        .store(true, Ordering::Relaxed);
}
