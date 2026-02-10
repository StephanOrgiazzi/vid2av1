mod cancellation;
mod encoder_cache;
mod process_registry;
mod termination;
mod types;

pub use cancellation::{cancel_active_conversion, clear_cancel_requested, is_cancel_requested};
pub use encoder_cache::{get_cached_av1_encoders, set_cached_av1_encoders};
pub use process_registry::{
    register_active_conversion, register_ffmpeg_pid, unregister_active_conversion,
    unregister_ffmpeg_pid,
};
pub use termination::terminate_all_active_ffmpeg;
pub use types::{ActiveConversionControl, ActiveFfmpegPids, Av1EncoderCache, GlobalCancelFlag};
