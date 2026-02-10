use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertRequest {
    pub input_path: String,
    #[serde(default)]
    pub av1_encoder: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertSummary {
    pub output_path: String,
    pub target_size_bytes: u64,
    pub audio_bitrate_kbps: u32,
    pub video_bitrate_kbps: u32,
    pub av1_encoder: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub percent: f64,
    pub speed: Option<f64>,
    pub eta_seconds: Option<f64>,
    pub label: String,
}
