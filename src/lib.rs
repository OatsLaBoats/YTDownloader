use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use iced::widget::image::Handle;

pub mod platform;
pub mod screen;
pub mod lang;
pub mod github;
pub mod command;
pub mod widget;

use lang::Language;

pub const VERSION: &'static str = "1.0.3";

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,

    #[default]
    Auto,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Auto => "Auto",
        };

        f.write_str(s)
    }
}

#[derive(Default, Clone, Copy, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum AudioConversionQuality {
    High,
    #[default]
    Medium,
    Low,
}

impl std::fmt::Display for AudioConversionQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        };

        f.write_str(s)
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SponsorBlockOption {
    pub sb_sponsor: bool,
    pub sb_intro: bool,
    pub sb_outro: bool,
    pub sb_selfpromo: bool,
    pub sb_preview: bool,
    pub sb_filler: bool,
    pub sb_interaction: bool,
    pub sb_music_offtopic: bool,
    pub sb_hook: bool,
    pub sb_chapter: bool,
    pub sb_all: bool,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub auto_updates: bool,
    pub ui_language: Language,
    pub ui_theme: Theme,

    // info_panel stuff
    pub audio_only: bool,
    pub conversion_quality: AudioConversionQuality,
    pub audio_format: command::yt_dlp::AudioFileType,
    pub video_format: command::yt_dlp::VideoFileType,
    pub download_dir: String,

    // Sponsorblock stuff
    pub sponsor_block: bool,
    pub sb_options: SponsorBlockOption,
}

#[derive(Default)]
pub struct Paths {
    pub downloads_dir: PathBuf,
    pub appdata_dir: PathBuf,
    pub downloader_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub yt_dlp_exe: PathBuf,
    pub ffmpeg_dir: PathBuf,
    pub ffmpeg_bin_dir: PathBuf,
    pub deno_exe: PathBuf,
    pub settings_file: PathBuf,

    pub video_dir: PathBuf,

    // temp paths for update and install
    pub tmp_dir: PathBuf,
    pub tmp_ffmpeg_dir: PathBuf,
    pub tmp_yt_dlp_exe: PathBuf,
    pub tmp_app_exe: PathBuf,

    pub old_yt_downloader_exe: PathBuf,
    pub old_yt_dlp_exe: PathBuf,
    pub old_ffmpeg_exe: PathBuf,
    pub old_deno_exe: PathBuf,
    pub old_version_file: PathBuf,
}

pub struct Images {
    pub paste: Handle,
    pub arrow_left: Handle,
    pub arrow_right: Handle,
    pub close: Handle,
    pub play: Handle,
    pub pause: Handle,
    pub download: Handle,
    pub folder: Handle,
}
