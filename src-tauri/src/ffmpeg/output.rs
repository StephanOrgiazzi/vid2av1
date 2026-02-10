use std::path::Path;

pub fn default_output_for_input(input_path: &str) -> Result<String, String> {
    let path = Path::new(input_path);
    let parent = path
        .parent()
        .ok_or_else(|| "Input path has no parent folder.".to_string())?;
    let stem = path
        .file_stem()
        .ok_or_else(|| "Input path has no file name.".to_string())?
        .to_string_lossy();
    Ok(parent
        .join(format!("{stem}.av1.mp4"))
        .to_string_lossy()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::default_output_for_input;

    #[test]
    fn default_output_for_input_replaces_extension() {
        let output =
            default_output_for_input("videos/input.mkv").expect("output path should be generated");
        assert!(output.ends_with("input.av1.mp4"));
    }
}
