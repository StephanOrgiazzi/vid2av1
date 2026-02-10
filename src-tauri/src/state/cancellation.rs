use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager};

use super::process_registry::set_active_conversion_cancel_requested;
use super::process_registry::{get_active_conversion, is_active_conversion_cancel_requested};
use super::termination::terminate_all_active_ffmpeg;
use super::types::GlobalCancelFlag;

pub fn cancel_active_conversion(app: &AppHandle) -> Result<(), String> {
    let control = get_active_conversion(app)?;
    request_cancel(app);
    control.mark_cancel_requested();
    let _ = control.send_quit_command();
    terminate_all_active_ffmpeg(app);
    Ok(())
}

pub fn is_cancel_requested(app: &AppHandle) -> bool {
    if app.state::<GlobalCancelFlag>().0.load(Ordering::Relaxed) {
        return true;
    }

    is_active_conversion_cancel_requested(app)
}

pub fn clear_cancel_requested(app: &AppHandle) {
    app.state::<GlobalCancelFlag>()
        .0
        .store(false, Ordering::Relaxed);
    set_active_conversion_cancel_requested(app, false);
}

pub fn request_cancel(app: &AppHandle) {
    app.state::<GlobalCancelFlag>()
        .0
        .store(true, Ordering::Relaxed);
}
