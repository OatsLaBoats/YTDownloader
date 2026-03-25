use iced::Element;
use iced::Task;
use iced::widget::*;

use crate::command::yt_dlp::{AudioFileType, AudioInfo, VideoFileType, VideoInfo, VideoQuality};
use crate::{AudioConversionQuality, SponsorBlockOption};

#[derive(Debug, Clone)]
pub enum DownloadInfo {
    Video {
        info: VideoInfo,
        selected_format: VideoFileType,
        selected_quality: VideoQuality,
        remux: bool,
        sb_options: Option<SponsorBlockOption>,
        download_location: String,
        link: String,
    },

    Audio {
        info: AudioInfo,
        conversion_quality: AudioConversionQuality,
        selected_format: AudioFileType,
        extract_from_video: bool,
        remux: bool,
        sb_options: Option<SponsorBlockOption>,
        download_location: String,
        link: String,
    },
}

pub enum MessageKind {
    
}

pub struct Message {
    pub id: usize,
    pub kind: MessageKind,
}

impl Message {
    pub fn new(id: usize, kind: MessageKind) -> Self {
        Self {
            id,
            kind,
        }
    }
}

pub enum ActionKind {
    None,
    Close,
    Run(Task<Message>),
}

pub struct Action {
    pub id: usize,
    pub kind: ActionKind,
}

impl Action {
    pub fn new(id: usize, kind: ActionKind) -> Self {
        Self {
            id,
            kind,
        }
    }

    pub fn none(id: usize) -> Self {
        Action::new(id, ActionKind::None)
    }

    pub fn close(id: usize) -> Self {
        Action::new(id, ActionKind::Close)
    }

    pub fn run(id: usize, task: Task<Message>) -> Self {
        Action::new(id, ActionKind::Run(task))
    }
}

pub struct State {
    id: usize,
    info: DownloadInfo,
}

impl State {
    pub fn new(id: usize, info: DownloadInfo) -> Self {
        Self {
            id,
            info,
        }
    }

    pub fn update(&mut self, message: MessageKind) -> Action {
        Action::none(self.id)
    }

    pub fn view(&self) -> Element<'_, Message> {
        space().into()
    }
}
