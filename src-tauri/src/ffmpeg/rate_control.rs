pub fn video_rate_args(video_bitrate_kbps: u32, strict_size: bool, encoder: &str) -> Vec<String> {
    let mut args = vec!["-b:v".to_string(), format!("{video_bitrate_kbps}k")];

    if strict_size {
        if encoder == "av1_nvenc" {
            args.push("-rc".to_string());
            args.push("cbr".to_string());
        }

        let buf = (video_bitrate_kbps * 2).max(1000);
        args.push("-minrate".to_string());
        args.push(format!("{video_bitrate_kbps}k"));
        args.push("-maxrate".to_string());
        args.push(format!("{video_bitrate_kbps}k"));
        args.push("-bufsize".to_string());
        args.push(format!("{buf}k"));
    }

    args
}
