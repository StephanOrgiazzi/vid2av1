use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;

use crate::encoder_service::get_available_av1_encoders;
use crate::ffmpeg::{
    default_output_for_input, read_audio_bitrate_kbps, read_ffprobe_duration,
    resolve_encoder_candidates, resolve_tool_path, video_rate_args,
};
use crate::model::ConvertRequest;

use super::cancellation::abort_if_cancel_requested;

const AUDIO_BITRATE_FALLBACK_KBPS: u32 = 128;
const STRICT_SIZE_DEFAULT: bool = true;
const MIN_VIDEO_BITRATE_KBPS: i64 = 100;

pub struct ConversionPlan {
    pub ffmpeg_path: PathBuf,
    pub output_path: PathBuf,
    pub duration_sec: f64,
    pub target_size_bytes: u64,
    pub audio_bitrate_kbps: u32,
    pub video_bitrate_kbps: u32,
    pub encoder_candidates: Vec<String>,
    input_path: String,
}

pub fn build_conversion_plan(
    app: &AppHandle,
    request: &ConvertRequest,
) -> Result<ConversionPlan, String> {
    abort_if_cancel_requested(app)?;

    let input_path = PathBuf::from(&request.input_path);
    if !input_path.exists() {
        return Err(format!(
            "Input file not found: {}",
            input_path.to_string_lossy()
        ));
    }

    let output_path = PathBuf::from(default_output_for_input(&request.input_path)?);
    let ffmpeg_path = resolve_tool_path(app, "ffmpeg.exe")?;
    let ffprobe_path = resolve_tool_path(app, "ffprobe.exe")?;

    abort_if_cancel_requested(app)?;
    let available_all = get_available_av1_encoders(app)?;
    let encoder_candidates =
        resolve_encoder_candidates(request.av1_encoder.as_deref(), &available_all)?;

    let input_size = fs::metadata(&input_path)
        .map_err(|e| format!("Could not read input file metadata: {e}"))?
        .len();
    let target_size_bytes = input_size / 2;

    let duration_sec = read_ffprobe_duration(&ffprobe_path, &request.input_path)?;
    abort_if_cancel_requested(app)?;

    let audio_bitrate_kbps = read_audio_bitrate_kbps(
        &ffprobe_path,
        &request.input_path,
        AUDIO_BITRATE_FALLBACK_KBPS,
    )?;
    abort_if_cancel_requested(app)?;

    let video_bitrate_kbps =
        compute_video_bitrate_kbps(target_size_bytes, duration_sec, audio_bitrate_kbps)?;

    Ok(ConversionPlan {
        ffmpeg_path,
        output_path,
        duration_sec,
        target_size_bytes,
        audio_bitrate_kbps,
        video_bitrate_kbps,
        encoder_candidates,
        input_path: request.input_path.clone(),
    })
}

pub fn build_encode_args(plan: &ConversionPlan, encoder: &str) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        plan.input_path.clone(),
        "-c:v".to_string(),
        encoder.to_string(),
    ];

    args.extend(video_rate_args(
        plan.video_bitrate_kbps,
        STRICT_SIZE_DEFAULT,
        encoder,
    ));
    args.extend(vec![
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        format!("{}k", plan.audio_bitrate_kbps),
        "-movflags".to_string(),
        "+faststart".to_string(),
        plan.output_path.to_string_lossy().to_string(),
    ]);

    args
}

fn compute_video_bitrate_kbps(
    target_size_bytes: u64,
    duration_sec: f64,
    audio_bitrate_kbps: u32,
) -> Result<u32, String> {
    let target_total_bitrate = ((target_size_bytes as f64 * 8.0) / duration_sec).floor();
    let target_total_kbps = (target_total_bitrate / 1000.0).floor() as i64;
    let video_bitrate_kbps = target_total_kbps - audio_bitrate_kbps as i64;

    if video_bitrate_kbps < MIN_VIDEO_BITRATE_KBPS {
        return Err(format!(
            "Computed video bitrate too low ({video_bitrate_kbps} kbps). Use a larger source file."
        ));
    }

    Ok(video_bitrate_kbps as u32)
}
