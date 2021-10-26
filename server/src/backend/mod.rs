use crate::errors::HomeRadioError;
mod file_backend;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use file_backend::*;



#[derive(Deserialize, Serialize)]
pub struct MediaSource {
    pub link: String,
    pub name: String,
    pub media_type: MediaType,
    pub currently_playing: Option<bool>,
    pub default_source: bool,
}

#[derive(Deserialize, Serialize)]
pub enum MediaType {
    Radio,
    YouTube,
}
