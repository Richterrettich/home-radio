use std::{io, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum HomeRadioError {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("error changeing volume to {0}")]
    VolumeChange(i32),
    #[error("error starting playback")]
    Play,
    #[error("unable to get media stream for url {0}")]
    NoMedia(String),
    #[error("media channel closed")]
    ChannelClosed
}
