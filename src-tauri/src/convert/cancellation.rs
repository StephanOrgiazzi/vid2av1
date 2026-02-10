use tauri::AppHandle;

use crate::state::is_cancel_requested;

pub const CANCELED_BY_USER_ERROR: &str = "Conversion canceled by user.";

pub fn abort_if_cancel_requested(app: &AppHandle) -> Result<(), String> {
    if is_cancel_requested(app) {
        return Err(CANCELED_BY_USER_ERROR.to_string());
    }

    Ok(())
}
