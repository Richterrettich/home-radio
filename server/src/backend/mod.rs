use crate::errors::HomeRadioError;
mod file_backend;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use file_backend::*;

#[async_trait]
pub trait Backend {
    async fn get_media_sources(&self) -> Result<Vec<MediaSource>, HomeRadioError>;
    async fn add_media_source(&self, source: MediaSource) -> Result<(), HomeRadioError>;
    async fn set_volume(&self, volume: u16) -> Result<(), HomeRadioError>;
    async fn get_volume(&self) -> Result<u16, HomeRadioError>;
}

#[derive(Deserialize, Serialize)]
pub struct MediaSource {
    pub link: String,
    pub name: String,
    pub media_type: MediaType,
}

#[derive(Deserialize, Serialize)]
pub enum MediaType {
    Radio,
    YouTube,
}
