use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("failed to spawn yt-dlp process")]
    SpawnYtDlpFailed(Arc<std::io::Error>),

    #[error("link is invalid")]
    InvalidLink,

    #[error("failed to convert bytes to utf8")]
    ConvertBytesToUTF8Failed,
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn query_version(yt_dlp_path: impl AsRef<OsStr>) -> Result<String> {
    let result = tokio::process::Command::new(yt_dlp_path)
        .arg("--version")
        .kill_on_drop(true)
        .output().await.map_err(|e|
            Error::SpawnYtDlpFailed(Arc::new(e))
        )?
        .stdout;

    
    
    // Cut the /r/n at the end
    let version_slice = str::from_utf8(&result).map_err(|_|
        Error::ConvertBytesToUTF8Failed
    )?.trim_end_matches(&['\r', '\n']);

    Ok(version_slice.to_string())
}

pub async fn query_link_info(
    force_ipv4: bool,
    yt_dlp_path: PathBuf,
    ffmpeg_path: PathBuf,
    deno_path: PathBuf,
    link: String,
) -> Result<LinkInfo> {
    let ipv4 = if force_ipv4 {
        tracing::info!("forcing ipv4");
        "--force-ipv4"
    } else {
        "--skip-unavailable-fragments"
    };
    
    // |_| is the separator to make parsing easier. It shoudln't conflict with the content of the query.
    let link_query = tokio::process::Command::new(&yt_dlp_path)
        .arg(ipv4)
        .arg("--ffmpeg-location")
        .arg(&ffmpeg_path)
        .arg("--js-runtimes")
        .arg(&deno_path)
        .arg("--flat-playlist")
        .arg("-I")
        .arg("1:1")
        .arg("--print")
        .arg("\
            OK\
            |_|%(playlist_id)s\
            |_|%(playlist_title)s\
            |_|%(playlist_count)s\
            |_|%(vcodec)s\
            |_|%(acodec)s\
            |_|%(title)s\
            |_|%(channel)s\
            |_|%(uploader)s\
            |_|%(creator)s\
            |_|\
        ")
        .arg(&link)
        .kill_on_drop(true)
        .output().await.map_err(|e|
            Error::SpawnYtDlpFailed(Arc::new(e))
        )?
        .stdout;

    let link_basic_info = str::from_utf8(&link_query).map_err(|_|
        Error::ConvertBytesToUTF8Failed
    )?;

    if !link_basic_info.starts_with("OK|_|") {
        return Err(Error::InvalidLink);
    }

    // parse basic info
    let mut it = link_basic_info.split("|_|");
    it.next();
    let playlist_id = it.next().unwrap();
    let playlist_name = it.next().unwrap();
    let playlist_count = it.next().unwrap();
    let vcodec = it.next().unwrap();
    let acodec = it.next().unwrap();
    let title = it.next().unwrap();
    let channel = it.next().unwrap();
    let uploader = it.next().unwrap();
    let creator = it.next().unwrap();

    // If it succeeds we know it's a playlist
    if playlist_id != "NA" && playlist_name != "NA" {
        let playlist_query = tokio::process::Command::new(&yt_dlp_path)
            .arg(ipv4)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg("--js-runtimes")
            .arg(&deno_path)
            .arg("--flat-playlist")
            .arg("--print")
            .arg("\
                OK\
                |_|%(title)s\
                |_|%(url)s\
                |_|\
            ")
            .arg(&link)
            .kill_on_drop(true)
            .output().await.map_err(|e|
                Error::SpawnYtDlpFailed(Arc::new(e))
            )?
            .stdout;

        let playlist_info = str::from_utf8(&playlist_query).map_err(|_|
            Error::ConvertBytesToUTF8Failed
        )?;

        if !playlist_info.starts_with("OK|_|") {
            return Err(Error::InvalidLink);
        }

        let name = playlist_name.to_string();
        let count = playlist_count.parse::<usize>().unwrap();

        let mut items = Vec::with_capacity(count);
        for line in playlist_info.lines() {
            let mut it = line.split("|_|");
            it.next(); // OK
            let title = it.next().unwrap();
            let url = it.next().unwrap();

            items.push(Arc::new(PlaylistItem {
                title: title.to_string(),
                url: url.to_string(),
            }));
        }

        return Ok(
            LinkInfo::Playlist(
                PlaylistInfo {
                    name,
                    items,
                },
            ),
        );
    } else if vcodec != "none" {
        let video_query = tokio::process::Command::new(&yt_dlp_path)
            .arg(ipv4)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg("--js-runtimes")
            .arg(&deno_path)
            .arg("--print")
            .arg("\
                OK\
                |_|%(format_id)s\
                |_|%(height>%s|NA)s\
                |_|%(fps>%s|NA)s\
                |_|%(ext)s\
                |_|%(resolution)s\
                |_|\
            ")
            .arg("-f")
            .arg("\
                bv*+ba[height=144]/b[height=144],\
                bv*+ba[height=240]/b[height=240],\
                bv*+ba[height=360]/b[height=360],\
                bv*+ba[height=480]/b[height=480],\
                bv*+ba[height=720]/b[height=720],\
                bv*+ba[height=1080]/b[height=1080],\
                bv*+ba[height=1440]/b[height=1440],\
                bv*+ba[height=2160]/b[height=2160],\
                ba,\
                bv*+ba/b,\
                ba*\
            ")
            .arg(&link)
            .kill_on_drop(true)
            .output().await.map_err(|e|
                Error::SpawnYtDlpFailed(Arc::new(e))
            )?
            .stdout;

        let video_info = str::from_utf8(&video_query).map_err(|_|
            Error::ConvertBytesToUTF8Failed
        )?;

        if !video_info.starts_with("OK|_|") {
            return Err(Error::InvalidLink);
        }

        let mut f_144 = None;
        let mut f_240 = None;
        let mut f_360 = None;
        let mut f_480 = None;
        let mut f_720 = None;
        let mut f_1080 = None;
        let mut f_1440 = None;
        let mut f_2160 = None;
        let mut f_best = None;
        let mut f_best_audio_only = None;
        let mut f_best_audio = None;

        for line in video_info.lines() {
            let mut it = line.split("|_|");
            it.next(); // OK
            let id = it.next().unwrap();
            let height = it.next().unwrap();
            let fps = it.next().unwrap();
            let ext = it.next().unwrap();
            let resolution = it.next().unwrap();

            // There order here is very intentional and important
            // any other order it will be parsed wrong
            if resolution == "audio only" {
                f_best_audio_only = Some(VideoFormat {
                    audio_only: true,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: None,
                    height: None,
                    resolution: None,
                })
            } else if height == "144" && f_144.is_none() {
                f_144 = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else if height == "240" && f_240.is_none() {
                f_240 = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else if height == "360" && f_360.is_none() {
                f_360 = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else if height == "480" && f_480.is_none() {
                f_480 = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else if height == "720" && f_720.is_none() {
                f_720 = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else if height == "1080" && f_1080.is_none() {
                f_1080 = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else if height == "1440" && f_1440.is_none() {
                f_1440 = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else if height == "2160" && f_2160.is_none() {
                f_2160 = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else if f_best.is_none() {
                f_best = Some(VideoFormat {
                    audio_only: false,
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: Some(resolution.to_string()),
                })
            } else {
                f_best_audio = Some(VideoFormat {
                    audio_only: resolution == "audio only",
                    format_id: id.to_string(),
                    file_ext: ext.to_string(),
                    fps: fps.parse().ok(),
                    height: height.parse().ok(),
                    resolution: if resolution == "audio only" {
                        None
                    } else {
                        Some(resolution.to_string())
                    },
                })
            }
        }
      
        let channel = if channel != "NA" {
            Some(channel.to_string())
        } else if uploader != "NA" {
            Some(uploader.to_string())
        } else if creator != "NA" {
            Some(creator.to_string())
        } else {
            None
        };

        let title = if title != "NA" {
            Some(title.to_string())
        } else {
            None
        };
        
        return Ok(
            LinkInfo::Video(
                VideoInfo {
                    title: title,
                    channel: channel,
                    f_144,
                    f_240,
                    f_360,
                    f_480,
                    f_720,
                    f_1080,
                    f_1440,
                    f_2160,
                    f_best,
                    f_best_audio_only,
                    f_best_audio,
                },
            ),
        );
    } else if acodec != "none" {
        let audio_query = tokio::process::Command::new(&yt_dlp_path)
            .arg(ipv4)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg("--js-runtimes")
            .arg(&deno_path)
            .arg("--print")
            .arg("\
                OK\
                |_|%(format_id)s\
                |_|%(ext)s\
                |_|\
            ")
            .arg("-f")
            .arg("\
                ba,\
                all\
            ")
            .arg(&link)
            .kill_on_drop(true)
            .output().await.map_err(|e|
                Error::SpawnYtDlpFailed(Arc::new(e))
            )?
            .stdout;

        let audio_info = str::from_utf8(&audio_query).map_err(|_|
            Error::ConvertBytesToUTF8Failed
        )?;

        if !audio_info.starts_with("OK|_|") {
            return Err(Error::InvalidLink);
        }

        let mut best = None;
        let mut formats = Vec::new();
        
        for line in audio_info.lines() {
            let mut it = line.split("|_|");
            it.next(); // OK
            let id = it.next().unwrap();
            let ext = it.next().unwrap();

            if best.is_none() {
                best = Some(
                    AudioFormat {
                        format_id: id.to_string(),
                        ext: ext.to_string(),
                    }
                );
            } else {
                formats.push(AudioFormat {
                    format_id: id.to_string(),
                    ext: ext.to_string(),
                });
            }
        }
        
        let channel = if channel != "NA" {
            Some(channel.to_string())
        } else if uploader != "NA" {
            Some(uploader.to_string())
        } else if creator != "NA" {
            Some(creator.to_string())
        } else {
            None
        };

        let title = if title != "NA" {
            Some(title.to_string())
        } else {
            None
        };

        return Ok(
            LinkInfo::Audio(
                AudioInfo {
                    title: title,
                    channel: channel,
                    f_best: best.unwrap(),
                    formats,
                },
            ),
        );
    }

    Err(Error::InvalidLink)
}

#[derive(Debug, Clone)]
pub enum LinkInfo {
    Playlist(PlaylistInfo),
    Video(VideoInfo),
    Audio(AudioInfo),
}

#[derive(Debug, Clone)]
pub struct PlaylistInfo {
    pub name: String,
    pub items: Vec<Arc<PlaylistItem>>, // Can not contain other playlists
}

#[derive(Debug, Clone)]
pub struct PlaylistItem {
    pub title: String,
    pub url: String,
}

impl std::fmt::Display for PlaylistItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.title)
    }
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub title: Option<String>,
    pub channel: Option<String>,

    // Formats
    pub f_144: Option<VideoFormat>,
    pub f_240: Option<VideoFormat>,
    pub f_360: Option<VideoFormat>,
    pub f_480: Option<VideoFormat>,
    pub f_720: Option<VideoFormat>,
    pub f_1080: Option<VideoFormat>,
    pub f_1440: Option<VideoFormat>,
    pub f_2160: Option<VideoFormat>,
    pub f_best: Option<VideoFormat>,
    pub f_best_audio_only: Option<VideoFormat>,
    pub f_best_audio: Option<VideoFormat>,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AudioFileType {
    #[default]
    MP3,
    OGG,
    WAV,
}

impl std::fmt::Display for AudioFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MP3 => "mp3",
            Self::OGG => "ogg",
            Self::WAV => "wav",
        };

        write!(f, "{s}")
    }
}

impl AudioFileType {
    pub fn file_types() -> [AudioFileType; 3] {
        [
            AudioFileType::MP3,
            AudioFileType::OGG,
            AudioFileType::WAV,
        ]
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VideoFileType {
    #[default]
    MP4,
    M4A,
    WEBM,
    FLV,
}

impl std::fmt::Display for VideoFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MP4 => "mp4",
            Self::M4A => "m4a",
            Self::WEBM => "webm",
            Self::FLV => "flv",
        };

        write!(f, "{s}")
    }
}

impl VideoFileType {
    pub fn file_types() -> [VideoFileType; 4] {
        [
            VideoFileType::MP4,
            VideoFileType::M4A,
            VideoFileType::WEBM,
            VideoFileType::FLV,
        ]
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum VideoQuality {
    Q144,
    Q240,
    Q360,
    Q480,
    Q720,
    Q1080,
    Q1440,
    Q2160,
    Best,
}

impl std::fmt::Display for VideoQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Q144 => "144p",
            Self::Q240 => "240p",
            Self::Q360 => "360p",
            Self::Q480 => "480p",
            Self::Q720 => "720p",
            Self::Q1080 => "1080p",
            Self::Q1440 => "1440p",
            Self::Q2160 => "2160p",
            Self::Best => "max",
        };
        
        write!(f, "{s}")
    }
}

impl VideoInfo {
    pub fn qualities(&self) -> Vec<VideoQuality> {
        let mut result = Vec::new();

        result.push(VideoQuality::Best);
        if self.f_2160.is_some() { result.push(VideoQuality::Q2160) }
        if self.f_1440.is_some() { result.push(VideoQuality::Q1440) }
        if self.f_1080.is_some() { result.push(VideoQuality::Q1080) }
        if self.f_720.is_some() { result.push(VideoQuality::Q720) }
        if self.f_480.is_some() { result.push(VideoQuality::Q480) }
        if self.f_360.is_some() { result.push(VideoQuality::Q360) }
        if self.f_240.is_some() { result.push(VideoQuality::Q240) }
        if self.f_144.is_some() { result.push(VideoQuality::Q144) }

        result
    }

    pub fn to_audio_info(&self) -> AudioInfo {
        AudioInfo {
            title: self.title.clone(),
            channel: self.channel.clone(),
            f_best: if let Some(f) = &self.f_best_audio_only {
                AudioFormat {
                    format_id: f.format_id.clone(),
                    ext: f.file_ext.clone(),
                }
            } else if let Some(f) = &self.f_best_audio {
                AudioFormat {
                    format_id: f.format_id.clone(),
                    ext: f.file_ext.clone(),
                }
            } else {
                AudioFormat {
                    format_id: self.f_best.as_ref().unwrap().format_id.clone(),
                    ext: self.f_best.as_ref().unwrap().file_ext.clone(),
                }
            },
            formats: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoFormat {
    pub audio_only: bool,
    pub format_id: String,
    pub file_ext: String,
    pub fps: Option<u32>,
    pub height: Option<u32>,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AudioInfo {
    pub title: Option<String>,
    pub channel: Option<String>,
    pub f_best: AudioFormat,
    pub formats: Vec<AudioFormat>,
}

#[derive(Debug, Clone)]
pub struct AudioFormat {
    pub format_id: String,
    pub ext: String,
}
