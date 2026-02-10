use tauri::{AppHandle, Manager};

use crate::convert::do_convert;
use crate::encoder_service::pick_auto_av1_encoder as pick_auto;
use crate::model::{ConvertRequest, ConvertSummary};
use crate::state::{cancel_active_conversion, clear_cancel_requested};

async fn run_blocking<T, F>(task_name: &str, task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|e| format!("{task_name} task failed: {e}"))?
}

#[tauri::command]
pub async fn pick_auto_av1_encoder(app: AppHandle) -> Result<String, String> {
    let app_clone = app.clone();
    run_blocking("Auto-encoder", move || pick_auto(&app_clone)).await
}

#[tauri::command]
pub async fn convert_video(
    app: AppHandle,
    request: ConvertRequest,
) -> Result<ConvertSummary, String> {
    clear_cancel_requested(&app);
    let app_clone = app.clone();
    run_blocking("Conversion", move || do_convert(&app_clone, request)).await
}

#[tauri::command]
pub fn cancel_conversion(app: AppHandle) -> Result<(), String> {
    cancel_active_conversion(&app)
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    reveal_main_window(&app)
}

pub fn reveal_main_window(app: &AppHandle) -> Result<(), String> {
    let Some(main_window) = app.get_webview_window("main") else {
        return Err("Main window not found.".to_string());
    };

    let _ = main_window.center();
    main_window
        .show()
        .map_err(|e| format!("Failed to show main window: {e}"))?;
    let _ = main_window.set_focus();
    Ok(())
}
