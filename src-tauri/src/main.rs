#![windows_subsystem = "windows"]

mod commands;
mod convert;
mod encoder_service;
mod ffmpeg;
mod model;
mod state;

use commands::{
    cancel_conversion, convert_video, pick_auto_av1_encoder, show_main_window,
};
use state::{
    terminate_all_active_ffmpeg, ActiveConversionControl, ActiveFfmpegPids, Av1EncoderCache,
    GlobalCancelFlag,
};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(ActiveFfmpegPids::default())
        .manage(ActiveConversionControl::default())
        .manage(Av1EncoderCache::default())
        .manage(GlobalCancelFlag::default())
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                terminate_all_active_ffmpeg(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            pick_auto_av1_encoder,
            convert_video,
            cancel_conversion,
            show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
