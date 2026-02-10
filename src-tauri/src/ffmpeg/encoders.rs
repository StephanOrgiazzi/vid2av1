use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use super::command::hidden_command;

pub const PREFERRED_ENCODER_ORDER: &[&str] = &[
    "av1_nvenc",
    "libsvtav1",
    "libaom-av1",
    "librav1e",
    "av1_qsv",
    "av1_amf",
    "av1_mf",
    "av1_vaapi",
];

pub fn list_encoders(ffmpeg_path: &Path) -> Result<Vec<String>, String> {
    let output = hidden_command(ffmpeg_path)
        .args(["-hide_banner", "-encoders"])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg -encoders: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "ffmpeg -encoders failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut encoders = BTreeSet::new();

    for line in text.lines() {
        if !line.contains("av1") {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() < 2 {
            continue;
        }

        if tokens[0].starts_with('V') {
            encoders.insert(tokens[1].to_string());
        }
    }

    Ok(encoders.into_iter().collect())
}

pub fn resolve_encoder_candidates(
    requested: Option<&str>,
    available_all: &[String],
) -> Result<Vec<String>, String> {
    if available_all.is_empty() {
        return Err("No supported AV1 encoder found in ffmpeg.".to_string());
    }

    let available_set: HashSet<&str> = available_all.iter().map(String::as_str).collect();
    let preferred_available = PREFERRED_ENCODER_ORDER
        .iter()
        .copied()
        .filter(|encoder| available_set.contains(*encoder));

    let mut ordered = Vec::new();
    let mut included = HashSet::new();

    let mut push_unique = |encoder: &str| {
        if included.insert(encoder.to_string()) {
            ordered.push(encoder.to_string());
        }
    };

    if let Some(requested_name) = requested {
        if available_set.contains(requested_name) {
            push_unique(requested_name);
        } else {
            return Err(format!(
                "Requested AV1 encoder is unavailable: {requested_name}"
            ));
        }
    }

    for encoder in preferred_available {
        push_unique(encoder);
    }

    for encoder in available_all {
        push_unique(encoder);
    }

    if ordered.is_empty() {
        return Err("No supported AV1 encoder found in ffmpeg.".to_string());
    }

    Ok(ordered)
}

#[cfg(test)]
mod tests {
    use super::resolve_encoder_candidates;

    #[test]
    fn resolve_encoder_candidates_prioritizes_requested_encoder() {
        let available = vec![
            "libsvtav1".to_string(),
            "av1_nvenc".to_string(),
            "my_custom_av1".to_string(),
        ];
        let ordered = resolve_encoder_candidates(Some("my_custom_av1"), &available)
            .expect("candidate resolution should succeed");

        assert_eq!(ordered.first().map(String::as_str), Some("my_custom_av1"));
        assert!(ordered.contains(&"av1_nvenc".to_string()));
        assert!(ordered.contains(&"libsvtav1".to_string()));
    }

    #[test]
    fn resolve_encoder_candidates_keeps_non_preferred_fallbacks() {
        let available = vec!["my_custom_av1".to_string()];
        let ordered = resolve_encoder_candidates(None, &available)
            .expect("candidate resolution should succeed");

        assert_eq!(ordered, vec!["my_custom_av1".to_string()]);
    }

    #[test]
    fn resolve_encoder_candidates_rejects_missing_requested_encoder() {
        let available = vec!["av1_nvenc".to_string()];
        let error = resolve_encoder_candidates(Some("libsvtav1"), &available)
            .expect_err("missing requested encoder should fail");

        assert!(error.contains("Requested AV1 encoder is unavailable"));
    }
}
