use std::path::Path;

use super::command::hidden_command;

pub fn read_ffprobe_duration(ffprobe_path: &Path, input_path: &str) -> Result<f64, String> {
    let output = hidden_command(ffprobe_path)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            input_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let duration = first_line_as_f64(&output.stdout)?;
    if duration <= 0.0 {
        return Err("Duration must be > 0.".to_string());
    }
    Ok(duration)
}

pub fn read_audio_bitrate_kbps(
    ffprobe_path: &Path,
    input_path: &str,
    fallback_kbps: u32,
) -> Result<u32, String> {
    let output = hidden_command(ffprobe_path)
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=bit_rate",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            input_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {e}"))?;

    if !output.status.success() {
        return Ok(fallback_kbps);
    }

    match first_line_as_f64(&output.stdout) {
        Ok(bit_rate) if bit_rate > 0.0 => Ok((bit_rate / 1000.0).floor() as u32),
        _ => Ok(fallback_kbps),
    }
}

fn first_line_as_f64(output: &[u8]) -> Result<f64, String> {
    let text = String::from_utf8_lossy(output);
    let line = text
        .lines()
        .find(|value| !value.trim().is_empty())
        .ok_or_else(|| "Expected ffprobe output.".to_string())?;
    line.trim()
        .parse::<f64>()
        .map_err(|e| format!("Parse error: {e}"))
}
