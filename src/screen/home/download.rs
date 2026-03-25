use crate::command::yt_dlp::{AudioFileType, AudioInfo, VideoFileType, VideoInfo, VideoQuality};

#[derive(Debug, Clone)]
pub enum DownloadInfo {
    Video {
        info: VideoInfo,
        selected_format: VideoFileType,
        selected_quality: VideoQuality,
    },

    Audio {
        info: AudioInfo,
        selected_format: AudioFileType,
        extract_from_video: bool,
    },
}

pub struct State {
    
}
