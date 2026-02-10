use std::collections::HashSet;
use std::io::Write;
use std::process::ChildStdin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ActiveFfmpegPids(pub Mutex<HashSet<u32>>);

#[derive(Default)]
pub struct Av1EncoderCache(pub Mutex<Option<Vec<String>>>);

#[derive(Default)]
pub struct ActiveConversionControl(pub Mutex<Option<Arc<ConversionControl>>>);

#[derive(Default)]
pub struct GlobalCancelFlag(pub AtomicBool);

pub struct ConversionControl {
    pid: u32,
    stdin: Mutex<ChildStdin>,
    cancel_requested: AtomicBool,
}

impl ConversionControl {
    pub fn new(pid: u32, stdin: ChildStdin) -> Self {
        Self {
            pid,
            stdin: Mutex::new(stdin),
            cancel_requested: AtomicBool::new(false),
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn mark_cancel_requested(&self) {
        self.cancel_requested.store(true, Ordering::Relaxed);
    }

    pub fn clear_cancel_requested(&self) {
        self.cancel_requested.store(false, Ordering::Relaxed);
    }

    pub fn is_cancel_requested(&self) -> bool {
        self.cancel_requested.load(Ordering::Relaxed)
    }

    pub fn send_quit_command(&self) -> Result<(), String> {
        self.send_stdin_command(b"q\n")
    }

    fn send_stdin_command(&self, command: &[u8]) -> Result<(), String> {
        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| "Failed to lock conversion stdin.".to_string())?;
        stdin
            .write_all(command)
            .map_err(|e| format!("Failed to send command to ffmpeg: {e}"))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush ffmpeg command: {e}"))?;
        Ok(())
    }
}
