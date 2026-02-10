use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::ffmpeg::hidden_command;
use crate::model::ProgressEvent;
use crate::state::{
    is_cancel_requested, register_active_conversion, register_ffmpeg_pid,
    unregister_active_conversion, unregister_ffmpeg_pid,
};

use super::cancellation::CANCELED_BY_USER_ERROR;

const PROGRESS_EMIT_INTERVAL: Duration = Duration::from_millis(120);
const PROGRESS_EMIT_PERCENT_STEP: f64 = 0.25;

struct FfmpegPidGuard {
    app: AppHandle,
    pid: u32,
}

impl Drop for FfmpegPidGuard {
    fn drop(&mut self) {
        unregister_ffmpeg_pid(&self.app, self.pid);
    }
}

struct ActiveConversionGuard {
    app: AppHandle,
    pid: u32,
}

impl Drop for ActiveConversionGuard {
    fn drop(&mut self) {
        unregister_active_conversion(&self.app, self.pid);
    }
}

pub fn run_ffmpeg_with_progress(
    app: &AppHandle,
    ffmpeg_path: &Path,
    args: &[String],
    duration_sec: f64,
    label: &str,
) -> Result<(), String> {
    let mut full_args = args.to_vec();
    full_args.extend_from_slice(&[
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
    ]);

    let mut child = hidden_command(ffmpeg_path)
        .args(full_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {e}"))?;

    let pid = child.id();
    register_ffmpeg_pid(app, pid)?;
    let _pid_guard = FfmpegPidGuard {
        app: app.clone(),
        pid,
    };

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "No ffmpeg stdin.".to_string())?;
    if let Err(error) = register_active_conversion(app, pid, stdin) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let _active_guard = ActiveConversionGuard {
        app: app.clone(),
        pid,
    };

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "No ffmpeg stdout.".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "No ffmpeg stderr.".to_string())?;

    let stderr_handle = thread::spawn(move || -> String {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        text
    });

    let mut out_time_sec = 0.0_f64;
    let mut speed = 0.0_f64;
    let mut last_emit_at = Instant::now()
        .checked_sub(PROGRESS_EMIT_INTERVAL)
        .unwrap_or_else(Instant::now);
    let mut last_emitted_percent = 0.0_f64;
    let label_owned = label.to_string();

    for line_result in BufReader::new(stdout).lines() {
        let line = line_result.map_err(|e| format!("Failed to read ffmpeg output: {e}"))?;
        if let Some(value) = line.strip_prefix("out_time_ms=") {
            if let Ok(ms) = value.trim().parse::<f64>() {
                out_time_sec = ms / 1_000_000.0;
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("speed=") {
            if let Some(raw_speed) = value.strip_suffix('x') {
                if let Ok(parsed_speed) = raw_speed.trim().parse::<f64>() {
                    speed = parsed_speed;
                }
            }
            continue;
        }

        if line.trim() != "progress=continue" || duration_sec <= 0.0 {
            continue;
        }

        let percent = (out_time_sec / duration_sec * 100.0).clamp(0.0, 100.0);
        let eta = if speed > 0.0 {
            Some(((duration_sec - out_time_sec).max(0.0)) / speed)
        } else {
            None
        };

        let now = Instant::now();
        let percent_advanced = percent - last_emitted_percent;
        let should_emit = percent >= 99.9
            || percent_advanced >= PROGRESS_EMIT_PERCENT_STEP
            || now.duration_since(last_emit_at) >= PROGRESS_EMIT_INTERVAL;
        if !should_emit {
            continue;
        }

        let payload = ProgressEvent {
            percent,
            speed: if speed > 0.0 { Some(speed) } else { None },
            eta_seconds: eta,
            label: label_owned.clone(),
        };
        app.emit("convert-progress", payload)
            .map_err(|e| format!("Failed to emit progress event: {e}"))?;
        last_emit_at = now;
        last_emitted_percent = percent;
    }

    let status = child
        .wait()
        .map_err(|e| format!("ffmpeg wait failed: {e}"))?;
    let stderr_text = stderr_handle
        .join()
        .unwrap_or_else(|_| "Failed to read ffmpeg stderr.".into());

    if !status.success() {
        if is_cancel_requested(app) {
            return Err(CANCELED_BY_USER_ERROR.to_string());
        }
        return Err(format!("ffmpeg failed: {stderr_text}"));
    }

    app.emit(
        "convert-progress",
        ProgressEvent {
            percent: 100.0,
            speed: Some(speed),
            eta_seconds: Some(0.0),
            label: label_owned,
        },
    )
    .map_err(|e| format!("Failed to emit progress event: {e}"))?;

    Ok(())
}
