#[cfg(not(windows))]
use std::process::Command;

use tauri::AppHandle;

use super::cancellation::request_cancel;
use super::process_registry::{
    clear_registered_ffmpeg_pids, collect_registered_ffmpeg_pids,
    set_active_conversion_cancel_requested,
};

fn terminate_pid(pid: u32) {
    #[cfg(windows)]
    {
        use crate::ffmpeg::hidden_program_command;
        let _ = hidden_program_command("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status();
    }

    #[cfg(not(windows))]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }
}

pub fn terminate_all_active_ffmpeg(app: &AppHandle) -> Result<(), String> {
    request_cancel(app);
    set_active_conversion_cancel_requested(app, true)?;

    let pids = collect_registered_ffmpeg_pids(app)?;
    for pid in pids {
        terminate_pid(pid);
    }

    clear_registered_ffmpeg_pids(app)?;
    Ok(())
}
