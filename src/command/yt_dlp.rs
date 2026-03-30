use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

use iced::task::{Straw, sipper};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, error};
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::{AudioConversionQuality, SponsorBlockOption};
use crate::platform::windows::convert_ascii_to_utf8;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("failed to spawn yt-dlp process")]
    SpawnYtDlpFailed(Arc<std::io::Error>),

    #[error("yt-dlp returned a non zero exit code")]
    YtDlpCommandFailed,

    #[error("link is invalid")]
    InvalidLink,

    #[error("failed to convert bytes to utf8")]
    ConvertBytesToUTF8Failed,
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn query_version(yt_dlp_path: impl AsRef<OsStr>) -> Result<String> {
    let result = tokio::process::Command::new(yt_dlp_path)
        .creation_flags(CREATE_NO_WINDOW.0)
        .arg("--version")
        .kill_on_drop(true)
        .output().await.map_err(|e|
            Error::SpawnYtDlpFailed(Arc::new(e))
        )?;
    
    let output = result.stdout;

    if !result.status.success() {
        return Err(Error::YtDlpCommandFailed);
    }
    
    // Cut the /r/n at the end
    let version_slice = convert_ascii_to_utf8(&output).ok_or(()).map_err(|_|
        Error::ConvertBytesToUTF8Failed
    )?;

    let version = version_slice.trim_end_matches(&['\r', '\n']);

    Ok(version.to_string())
}

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Starting(Option<u32>),
    Downloading(ProgressDownloading),
}

#[derive(Debug, Clone)]
pub struct ProgressDownloading {
    pub tmp_file_name: Option<String>,
    pub eta: Option<u32>,
    pub download_speed: Option<usize>,
    pub downloaded_bytes: Option<usize>,
    pub total_bytes: Option<usize>,
    pub total_bytes_estimate: Option<f64>,
    pub percent: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum DownloadParams {
    Video(VideoDownloadParams),
    Audio(AudioDownloadParams),
}

#[derive(Debug, Clone)]
pub struct VideoDownloadParams {
    pub quality: VideoQuality,
    pub format: VideoFileType,
    pub sb_options: Option<SponsorBlockOption>,
}

#[derive(Debug, Clone)]
pub struct AudioDownloadParams {
    pub quality: AudioConversionQuality,
    pub format: AudioFileType,
    pub sb_options: Option<SponsorBlockOption>,
}

pub fn download_media(
    id: u64,
    video_dir: PathBuf,

    yt_dlp_path: PathBuf,
    ffmpeg_path: PathBuf,
    deno_path: PathBuf,

    output_path: String,
    link: String,
    force_ipv4: bool,
    params: DownloadParams,
) -> impl Straw<(), DownloadProgress, ()> {
    sipper(async move |mut progress| {
        progress.send(DownloadProgress::Starting(None)).await;

        if !video_dir.exists() {
            tokio::fs::create_dir(&video_dir).await
                .map_err(|e| error!("DOWNLOAD_MEDIA: failed to create video directory -> {e}"))?;
        }

        let mut download_path = video_dir.clone();
        download_path.push(format!("dl{id}"));

        if !download_path.exists() {
            tokio::fs::create_dir(&download_path).await
                .map_err(|e| error!("DOWNLOAD_MEDIA: failed to create video download directory -> {e}"))?;
        }

        let path_str = download_path.to_string_lossy();
       
        let mut command = tokio::process::Command::new(&yt_dlp_path);
        command
            .creation_flags(CREATE_NO_WINDOW.0)
            .arg(&link)
            .arg("--no-playlist")
            .arg("--no-mtime")
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg("--no-js-runtimes")
            .arg("--js-runtimes")
            .arg(format!("deno:{}", deno_path.to_string_lossy()))
            .arg("--newline")
            .arg("-q")
            .arg("--progress")
            .arg("--progress-delta")
            .arg("0.2")
            .arg("--progress-template")
            .arg("\
                OK\
                |_|%(progress.tmpfilename)s\
                |_|%(progress.eta)s\
                |_|%(progress.speed)s\
                |_|%(progress.total_bytes_estimate)s\
                |_|%(progress.total_bytes)s\
                |_|%(progress.downloaded_bytes)s\
                |_|%(progress._percent)s\
                |_|\
            ")
            .arg("-o")
            .arg(format!("{path_str}\\%(title)s.%(ext)s"))
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped());

        if force_ipv4 {
            info!("DOWNLOAD_MEDIA: forcing ipv4");
            command.arg("--force-ipv4");
        }

        match params {
            DownloadParams::Video(v) => {
                if let Some(s) = v.sb_options {
                    let mut opts = String::new();
                    if s.sb_sponsor {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("sponsor");
                    }
                    if s.sb_intro {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("intro");
                    }
                    if s.sb_outro {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("outro");
                    }
                    if s.sb_selfpromo {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("selfpromo");
                    }
                    if s.sb_preview {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("preview");
                    }
                    if s.sb_filler {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("filler");
                    }
                    if s.sb_interaction {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("interaction");
                    }
                    if s.sb_music_offtopic {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("music_offtopic");
                    }

                    if s.sb_hook {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("hook");
                    }

                    if s.sb_chapter {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("chapter");
                    }

                    if s.sb_all {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("all");
                    }

                    if !opts.is_empty() {
                        command
                            .arg("--sponsorblock-remove")
                            .arg(&opts);
                    }
                }

                let mut quality = v.quality.to_string();
                quality.pop(); // Remove the 'p'

                if v.format == VideoFileType::Best {
                    if v.quality == VideoQuality::Best {
                        command
                            .arg("-f")
                            .arg(format!("bv*+ba/b"));
                    } else {
                        command
                            .arg("-f")
                            .arg(format!("bv*[height={}]+ba/b[height={}]", quality, quality));
                    }
                } else {
                    if v.quality == VideoQuality::Best {
                        command
                            .arg("-f")
                            .arg(format!("bv*[ext={}]+ba/b", v.format));
                    } else {
                        command
                            .arg("-f")
                            .arg(format!("bv*[height={}][ext={}]+ba/b[height={}]", quality, v.format, quality));
                    }
                }

                if v.format != VideoFileType::Best {
                    command
                        .arg("--recode-video")
                        .arg(format!("{}", v.format));
                }
            },

            DownloadParams::Audio(v) => {
                if let Some(s) = v.sb_options {
                    let mut opts = String::new();
                    if s.sb_sponsor {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("sponsor");
                    }
                    if s.sb_intro {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("intro");
                    }
                    if s.sb_outro {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("outro");
                    }
                    if s.sb_selfpromo {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("selfpromo");
                    }
                    if s.sb_preview {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("preview");
                    }
                    if s.sb_filler {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("filler");
                    }
                    if s.sb_interaction {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("interaction");
                    }
                    if s.sb_music_offtopic {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("music_offtopic");
                    }

                    if s.sb_hook {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("hook");
                    }

                    if s.sb_chapter {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("chapter");
                    }

                    if s.sb_all {
                        if !opts.is_empty() {
                            opts.push(',');
                        }
                        opts.push_str("all");
                    }

                    if !opts.is_empty() {
                        command
                            .arg("--sponsorblock-remove")
                            .arg(&opts);
                    }
                }

                command
                    .arg("-f")
                    .arg("ba/ba*/b");

                if v.format == AudioFileType::OGG || v.format == AudioFileType::AIFF || v.format == AudioFileType::MKA {
                    command
                        .arg("--recode-video")
                        .arg(format!("{}", v.format));
                } else {
                    let quality = match v.quality {
                        AudioConversionQuality::High => "0",
                        AudioConversionQuality::Medium => "5",
                        AudioConversionQuality::Low => "10",
                    };

                    command
                        .arg("--extract-audio")
                        .arg("--audio-quality")
                        .arg(quality)
                        .arg("--audio-format")
                        .arg(format!("{}", v.format));
                }
            },
        }

        let mut child = command.spawn().map_err(|e| {
                error!("DOWNLOAD_MEDIA: failed to spawn yt-dlp process -> {e}");
                ()
            })?;

        progress.send(DownloadProgress::Starting(child.id())).await;

        let Some(stdout) = child.stdout.take() else {
            error!("DOWNLOAD_MEDIA: failed to take ownership of yt-dlp stdout");
            return Err(());
        };

        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await.map_err(|e| {
            error!("DOWNLOAD_MEDIA: failed to read the next line of the yt-dlp child process -> {e}");
            ()
        })? {
            if !line.starts_with("OK|_|") { continue }
            let mut it = line.split("|_|");
            it.next();
            let temp_file_name = it.next().unwrap_or("NA");
            let eta = it.next().unwrap_or("NA");
            let speed = it.next().unwrap_or("NA");
            let total_bytes_estimate = it.next().unwrap_or("NA");
            let total_bytes = it.next().unwrap_or("NA");
            let downloaded_bytes = it.next().unwrap_or("NA");
            let percent = it.next().unwrap_or("NA");

            let tmp_file_name = if temp_file_name != "NA" {
                Some(temp_file_name.to_string())
            } else {
                None
            };

            progress.send(DownloadProgress::Downloading(ProgressDownloading {
                tmp_file_name,
                eta: eta.parse().ok(),
                download_speed: speed.parse().ok(),
                total_bytes: total_bytes.parse().ok(),
                total_bytes_estimate: total_bytes_estimate.parse().ok(),
                downloaded_bytes: downloaded_bytes.parse().ok(),
                percent: percent.parse().ok(),
            })).await;
        }

        // Move downloaded files
        let dl_path = download_path.clone();
        tokio::task::block_in_place(move || -> std::result::Result<(), ()> {
            let paths = std::fs::read_dir(&dl_path)
                .map_err(|e| error!("DOWNLOAD_MEDIA: failed to read video download directory -> {e}"))?
                .filter_map(|r| r.ok())
                .map(|de| de.path())
                .collect::<Vec<_>>();

            for p in paths {
                let filename = p.file_name()
                    .ok_or(())
                    .map_err(|_| error!("DOWNLOAD_MEDIA: failed to get result filename"))?
                    .to_string_lossy();

                std::fs::copy(&p, format!("{output_path}\\{filename}"))
                    .map_err(|e| error!("DOWNLOAD_MEDIA: failed to copy result video file -> {e}"))?;
            }

            Ok(())
        })?;

        tokio::fs::remove_dir_all(&download_path).await
            .map_err(|e| error!("DOWNLOAD_MEDIA: failed to remove video download directory -> {e}"))?;

        match child.wait().await {
            Ok(s) => {
                info!("DOWNLOAD_MEDIA: yt-dlp child process exited with status: {s}");
                Ok(())
            },
            
            Err(e) => {
                error!("DOWNLOAD_MEDIA: yt-dlp child process encountered and error -> {e}");
                Err(())
            },
        }
    })
}

pub async fn query_link_info(
    force_ipv4: bool,
    yt_dlp_path: PathBuf,
    ffmpeg_path: PathBuf,
    deno_path: PathBuf,
    link: String,
) -> Result<LinkInfo> {
    let ipv4 = if force_ipv4 {
        info!("forcing ipv4");
        "--force-ipv4" // Useful for when being rate limited
    } else {
        "--skip-unavailable-fragments" // On by default so we use it as a place holder
    };
    
    // |_| is the separator to make parsing easier. It shoudln't conflict with the content of the query.
    let link_query_result = tokio::process::Command::new(&yt_dlp_path)
        .creation_flags(CREATE_NO_WINDOW.0)
        .arg(ipv4)
        .arg("--ffmpeg-location")
        .arg(&ffmpeg_path)
        .arg("--js-runtimes")
        .arg(format!("deno:{}", deno_path.to_string_lossy()))
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
        )?;

    let link_query = link_query_result.stdout;
    if !link_query_result.status.success() {
        return Err(Error::YtDlpCommandFailed);
    }

    info!("QUERY_LINK_INFO: query done {}", link_query.len());

    let link_basic_info = convert_ascii_to_utf8(&link_query).ok_or(()).map_err(|_| {
        error!("failed to convert bytes to utf8");
        Error::ConvertBytesToUTF8Failed
    })?;

    info!("QUERY_LINK_INFO: bytes are valid");

    if !link_basic_info.starts_with("OK|_|") {
        return Err(Error::InvalidLink);
    }

    // parse basic info
    let mut it = link_basic_info.split("|_|");
    it.next();
    let playlist_id = it.next().unwrap_or("NA");
    let playlist_name = it.next().unwrap_or("NA");
    let playlist_count = it.next().unwrap_or("NA");
    let vcodec = it.next().unwrap_or("NA");
    let acodec = it.next().unwrap_or("NA");
    let title = it.next().unwrap_or("NA");
    let channel = it.next().unwrap_or("NA");
    let uploader = it.next().unwrap_or("NA");
    let creator = it.next().unwrap_or("NA");

    info!("QUERY_LINK_INFO: parsed basic info");

    // If it succeeds we know it's a playlist
    if playlist_id != "NA" || playlist_name != "NA" {
        info!("QUERY_LINK_INFO: getting playlist info");
        let playlist_query_result = tokio::process::Command::new(&yt_dlp_path)
            .creation_flags(CREATE_NO_WINDOW.0)
            .arg(ipv4)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg("--js-runtimes")
            .arg(format!("deno:{}", deno_path.to_string_lossy()))
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
            )?;

        let playlist_query = playlist_query_result.stdout;
        if !playlist_query_result.status.success() {
            return Err(Error::YtDlpCommandFailed);
        }

        info!("QUERY_LINK_INFO: query done {}", playlist_query.len());

        let playlist_info = convert_ascii_to_utf8(&playlist_query).ok_or(()).map_err(|_| {
            error!("QUERY_LINK_INFO: failed to convert bytes");
            Error::ConvertBytesToUTF8Failed
        })?;

        if !playlist_info.starts_with("OK|_|") {
            return Err(Error::InvalidLink);
        }

        let count = playlist_count.parse::<usize>().unwrap_or(0);

        let name = if playlist_name != "NA" {
            Some(playlist_name.to_string())
        } else {
            None
        };

        let mut items = Vec::with_capacity(count);
        for (i, line) in playlist_info.lines().enumerate() {
            let mut it = line.split("|_|");
            it.next(); // OK
            let title = it.next().unwrap_or("NA");
            let url = it.next().unwrap_or("NA");

            let title = if title != "NA" {
                Some(title.to_string())
            } else {
                None
            };

            items.push(Arc::new(PlaylistItem {
                title: title,
                url: url.to_string(),
                index: i + 1,
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
    } else if vcodec != "none" && vcodec != "NA" {
        let video_query_result = tokio::process::Command::new(&yt_dlp_path)
            .creation_flags(CREATE_NO_WINDOW.0)
            .arg(ipv4)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg("--js-runtimes")
            .arg(format!("deno:{}", deno_path.to_string_lossy()))
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
            )?;

        let video_query = video_query_result.stdout;
        if !video_query_result.status.success() {
            return Err(Error::YtDlpCommandFailed);
        }

        let video_info = convert_ascii_to_utf8(&video_query).ok_or(()).map_err(|_|
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
            let id = it.next().unwrap_or("NA");
            let height = it.next().unwrap_or("NA");
            let fps = it.next().unwrap_or("NA");
            let ext = it.next().unwrap_or("NA");
            let resolution = it.next().unwrap_or("NA");

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
                    has_audio: acodec != "none",
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
    } else if acodec != "none" && acodec != "NA" {
        let audio_query_result = tokio::process::Command::new(&yt_dlp_path)
            .creation_flags(CREATE_NO_WINDOW.0)
            .arg(ipv4)
            .arg("--ffmpeg-location")
            .arg(&ffmpeg_path)
            .arg("--js-runtimes")
            .arg(format!("deno:{}", deno_path.to_string_lossy()))
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
            )?;

        let audio_query = audio_query_result.stdout;
        if !audio_query_result.status.success() {
            return Err(Error::YtDlpCommandFailed);
        }

        let audio_info = convert_ascii_to_utf8(&audio_query).ok_or(()).map_err(|_|
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
            let id = it.next().unwrap_or("NA");
            let ext = it.next().unwrap_or("NA");

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
    pub name: Option<String>,
    pub items: Vec<Arc<PlaylistItem>>, // Can not contain other playlists
}

#[derive(Debug, Clone)]
pub struct PlaylistItem {
    pub title: Option<String>,
    pub url: String,
    pub index: usize,
}

impl std::fmt::Display for PlaylistItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.title {
            return f.write_str(&v);
        }

        f.write_str(&format!("{}", self.index))
    }
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub title: Option<String>,
    pub channel: Option<String>,
    pub has_audio: bool,

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
    OGG, // Can only be remuxed
    WAV,
    ACC,
    ALAC,
    FLAC,
    M4A,
    OPUS,
    VORBIS,
    AIFF, // Can't be extracted has to be remuxed and recoded
    MKA, // Same as above
    BEST,
}

impl std::fmt::Display for AudioFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MP3 => "mp3",
            Self::OGG => "ogg",
            Self::WAV => "wav",
            Self::ACC => "acc",
            Self::ALAC => "alac",
            Self::FLAC => "flac",
            Self::M4A => "m4a",
            Self::OPUS => "opus",
            Self::VORBIS => "vorbis",
            Self::AIFF => "aiff",
            Self::MKA => "mka",
            Self::BEST => "best",
        };

        write!(f, "{s}")
    }
}

impl AudioFileType {
    pub fn file_types() -> [AudioFileType; 12] {
        [
            AudioFileType::MP3,
            AudioFileType::OGG,
            AudioFileType::WAV,
            AudioFileType::ACC,
            AudioFileType::ALAC,
            AudioFileType::FLAC,
            AudioFileType::M4A,
            AudioFileType::OPUS,
            AudioFileType::VORBIS,
            AudioFileType::AIFF,
            AudioFileType::MKA,
            AudioFileType::BEST,
        ]
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VideoFileType {
    #[default]
    MP4,
    WEBM,
    FLV,
    AVI,
    GIF,
    MKV,
    MOV,
    Best,
}

impl std::fmt::Display for VideoFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MP4 => "mp4",
            Self::WEBM => "webm",
            Self::FLV => "flv",
            Self::AVI => "avi",
            Self::GIF => "gif",
            Self::MKV => "mkv",
            Self::MOV => "mov",
            Self::Best => "best",
        };

        write!(f, "{s}")
    }
}

impl VideoFileType {
    pub fn file_types() -> [VideoFileType; 8] {
        [
            VideoFileType::MP4,
            VideoFileType::WEBM,
            VideoFileType::FLV,
            VideoFileType::AVI,
            VideoFileType::GIF,
            VideoFileType::MKV,
            VideoFileType::MOV,
            VideoFileType::Best,
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
            Self::Best => "best",
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
