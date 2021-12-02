use std::{io, num::ParseIntError};

use awc;
use awc::error::SendRequestError;
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

    #[error(transparent)]
    SendRequestError(#[from] SendRequestError),
    #[error(transparent)]
    UrlEncodedError(Box<dyn std::error::Error>),

    #[error(transparent)]
    JsonPayloadError(#[from] awc::error::JsonPayloadError),
    #[error(transparent)]
    PayloadError(#[from] awc::error::PayloadError),

    #[error("internal vlc server is unhealthy")]
    VLCServerUnhealthy,
}
