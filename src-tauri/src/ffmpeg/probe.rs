use std::path::Path;

use super::command::hidden_command;

pub struct ProbeMetadata {
    pub duration_sec: f64,
    pub audio_bitrate_kbps: u32,
}

pub fn read_probe_metadata(
    ffprobe_path: &Path,
    input_path: &str,
    fallback_audio_kbps: u32,
) -> Result<ProbeMetadata, String> {
    let output = hidden_command(ffprobe_path)
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "format=duration:stream=bit_rate",
            "-of",
            "default=noprint_wrappers=1",
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

    let text = String::from_utf8_lossy(&output.stdout);
    let duration = find_value(&text, "duration")
        .and_then(|value| value.parse::<f64>().ok())
        .ok_or_else(|| "Expected ffprobe duration output.".to_string())?;
    if duration <= 0.0 {
        return Err("Duration must be > 0.".to_string());
    }

    let audio_bitrate_kbps = find_value(&text, "bit_rate")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|bit_rate| *bit_rate > 0.0)
        .map(|bit_rate| (bit_rate / 1000.0).floor() as u32)
        .unwrap_or(fallback_audio_kbps);

    Ok(ProbeMetadata {
        duration_sec: duration,
        audio_bitrate_kbps,
    })
}

fn find_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some((found_key, value)) = trimmed.split_once('=') {
            if found_key == key {
                return Some(value.trim());
            }
        }
    }

    None
}
