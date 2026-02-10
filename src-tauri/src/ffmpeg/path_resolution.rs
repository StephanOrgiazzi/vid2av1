use std::collections::HashSet;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

pub fn resolve_tool_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let candidates = collect_tool_path_candidates(app, filename);
    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!("Could not find {filename}."))
}

fn collect_tool_path_candidates(app: &AppHandle, filename: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut seen_paths = HashSet::new();

    let mut push_candidate = |candidate: PathBuf| {
        if seen_paths.insert(candidate.clone()) {
            candidates.push(candidate);
        }
    };

    if let Ok(resource_dir) = app.path().resource_dir() {
        push_candidate(
            resource_dir
                .join("vendor")
                .join("ffmpeg")
                .join("bin")
                .join(filename),
        );
        push_candidate(resource_dir.join("ffmpeg").join("bin").join(filename));
        push_candidate(resource_dir.join(filename));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            push_candidate(
                exe_dir
                    .join("vendor")
                    .join("ffmpeg")
                    .join("bin")
                    .join(filename),
            );
            push_candidate(exe_dir.join(filename));
            if let Some(parent) = exe_dir.parent() {
                push_candidate(
                    parent
                        .join("vendor")
                        .join("ffmpeg")
                        .join("bin")
                        .join(filename),
                );
            }
        }
    }

    push_candidate(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("vendor")
            .join("ffmpeg")
            .join("bin")
            .join(filename),
    );

    if let Ok(cwd) = std::env::current_dir() {
        push_candidate(cwd.join("vendor").join("ffmpeg").join("bin").join(filename));
    }

    candidates
}
