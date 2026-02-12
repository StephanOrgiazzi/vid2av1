use tauri::AppHandle;

use crate::error_protocol::{error_canceled_by_user, is_error_code, ERROR_CODE_CANCELED_BY_USER};
use crate::state::is_cancel_requested;

pub fn abort_if_cancel_requested(app: &AppHandle) -> Result<(), String> {
    if is_cancel_requested(app) {
        return Err(error_canceled_by_user());
    }

    Ok(())
}

pub fn is_canceled_by_user_error(error: &str) -> bool {
    is_error_code(error, ERROR_CODE_CANCELED_BY_USER)
}
