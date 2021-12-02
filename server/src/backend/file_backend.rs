use std::path::{Path, PathBuf};

use log::info;
use tokio::fs::{self, File, OpenOptions};

use crate::errors::HomeRadioError;

use serde_json;
use tokio::io::AsyncWriteExt;

pub struct FileBackend {
    media_file_path: PathBuf,
    volume_path: PathBuf,
    currently_playing_path: PathBuf,
}

impl FileBackend {
    pub async fn new<P: AsRef<Path>>(dir: P) -> Result<Self, HomeRadioError> {
        let dir = dir.as_ref();
        info!("using dir {}", &dir.to_string_lossy());
        tokio::fs::create_dir_all(dir).await?;
        let mut media_sources_file = dir.to_path_buf();
        let mut volume_file_path = media_sources_file.clone();
        let mut currently_playing_path = media_sources_file.clone();

        media_sources_file.push("media-sources.json");
        volume_file_path.push("volume");
        currently_playing_path.push("currently-playing");

        for i in [
            &media_sources_file,
            &volume_file_path,
            &currently_playing_path,
        ] {
            info!("checking file {}", &i.to_string_lossy());
            let result = OpenOptions::new()
                .create_new(true)
                .write(true)
                .truncate(false)
                .open(i)
                .await;

            if let Err(e) = result {
                match e.kind() {
                    std::io::ErrorKind::AlreadyExists => {
                        // do nothing since it already is present
                    }
                    _ => return Err(HomeRadioError::Io(e)),
                }
            }
        }
        Ok(FileBackend {
            media_file_path: media_sources_file,
            volume_path: volume_file_path,
            currently_playing_path,
        })
    }

    pub async fn get_media_sources(
        &self,
    ) -> Result<Vec<super::MediaSource>, crate::errors::HomeRadioError> {
        let content = tokio::fs::read_to_string(&self.media_file_path).await?;
        if content.is_empty() {
            return Ok(Vec::new());
        }
        let result = serde_json::from_str(&content)?;
        return Ok(result);
    }

    pub async fn add_media_source(
        &self,
        source: super::MediaSource,
    ) -> Result<(), crate::errors::HomeRadioError> {
        let mut sources = self.get_media_sources().await?;

        let prev_source = sources.iter_mut().find(|m| &m.name == &source.name);
        if let Some(prev_source) = prev_source {
            *prev_source = source;
        } else {
            sources.push(source);
        }

        let mut f = File::create(&self.media_file_path).await?;

        let raw = serde_json::to_vec_pretty(&sources)?;
        f.write(&raw).await?;

        Ok(())
    }

    pub async fn set_volume(&self, volume: u16) -> Result<(), crate::errors::HomeRadioError> {
        tokio::fs::write(&self.volume_path, volume.to_string()).await?;
        Ok(())
    }

    pub async fn get_volume(&self) -> Result<u16, crate::errors::HomeRadioError> {
        let raw_vol = tokio::fs::read_to_string(&self.volume_path).await?;
        if raw_vol.is_empty() {
            return Ok(100);
        }
        Ok(raw_vol.parse()?)
    }

    pub async fn remove_current_media_source(&self) -> Result<(), HomeRadioError> {
        let result = fs::remove_file(&self.currently_playing_path).await;
        if let Err(e) = result {
            match e.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => return Err(HomeRadioError::Io(e)),
            }
        }
        Ok(())
    }

    pub async fn get_current_media_source(&self) -> Result<Option<String>, HomeRadioError> {
        let result = fs::read_to_string(&self.currently_playing_path).await;
        if let Ok(url) = result {
            if url.is_empty() {
                return Ok(None);
            }
            return Ok(Some(url));
        }
        let err = result.err().unwrap();
        match err.kind() {
            std::io::ErrorKind::NotFound => return Ok(None),
            _ => return Err(HomeRadioError::Io(err)),
        }
    }

    pub async fn set_current_media_source(&self, url: &str) -> Result<(), HomeRadioError> {
        fs::write(&self.currently_playing_path, &url).await?;
        Ok(())
    }
}
