mod cancellation;
mod ffmpeg_runner;
mod planning;

use tauri::AppHandle;

use crate::model::{ConvertRequest, ConvertSummary};

use self::cancellation::{abort_if_cancel_requested, is_canceled_by_user_error};
use self::ffmpeg_runner::run_ffmpeg_with_progress;
use self::planning::{build_conversion_plan, build_encode_args};

pub fn do_convert(app: &AppHandle, request: ConvertRequest) -> Result<ConvertSummary, String> {
    let plan = build_conversion_plan(app, &request)?;

    let mut selected_encoder: Option<String> = None;
    let mut last_error = String::new();

    for encoder in &plan.encoder_candidates {
        abort_if_cancel_requested(app)?;

        let args = build_encode_args(&plan, encoder);
        match run_ffmpeg_with_progress(app, &plan.ffmpeg_path, &args, plan.duration_sec, "Encode") {
            Ok(()) => {
                selected_encoder = Some(encoder.clone());
                break;
            }
            Err(error) => {
                if is_canceled_by_user_error(&error) {
                    return Err(error);
                }
                last_error = format!("{encoder}: {error}");
            }
        }
    }

    let selected_encoder = selected_encoder.ok_or_else(|| {
        format!(
            "All AV1 encoder attempts failed ({}). Last error: {}",
            plan.encoder_candidates.join(", "),
            last_error
        )
    })?;

    Ok(ConvertSummary {
        output_path: plan.output_path.to_string_lossy().to_string(),
        target_size_bytes: plan.target_size_bytes,
        audio_bitrate_kbps: plan.audio_bitrate_kbps,
        video_bitrate_kbps: plan.video_bitrate_kbps,
        av1_encoder: selected_encoder,
    })
}
