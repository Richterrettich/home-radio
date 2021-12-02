use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use tokio::{self, time::sleep};

use crate::errors::HomeRadioError;

#[derive(Clone)]
pub struct RemoteMediaService {
    base_url: String,
    host: String,
    port: String,
    client: awc::Client,
}

impl RemoteMediaService {
    pub fn new_with_auth(host: String, port: String, pwd: String) -> Self {
        let client = awc::Client::builder()
            .basic_auth("", Some(&pwd[..]))
            .finish();
        let base_url = format!("http://{}:{}", host, port);
        RemoteMediaService {
            host,
            port,
            client,
            base_url,
        }
    }
    async fn remote_command(
        &self,
        command: &str,
        query: &[(&str, &str)],
    ) -> Result<(), HomeRadioError> {
        let mut map = HashMap::new();

        for (k, v) in query {
            map.insert(*k, *v);
        }
        map.insert("command", command);
        self.client
            .get(format!("{}/requests/status.json", self.base_url))
            .query(&map)
            .map_err(|e| HomeRadioError::UrlEncodedError(Box::new(e)))?
            .send()
            .await?;
        Ok(())
    }

    pub async fn play(&self, url: &str, volume: u16) -> Result<(), HomeRadioError> {
        self.remote_command("pl_empty", &[]).await?;
        self.remote_command("in_play", &[("input", url), ("option", "novideo")])
            .await?;
        let state = self.get_status().await?.state;
        if &state == "stopped" {
            loop {
                let status = self.get_status().await?;
                dbg!(&status.state);
                if &status.state[..] == "playing" {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            self.set_volume(volume).await?;
        }

        Ok(())
    }

    pub async fn wait_for_healthy(
        &self,
        max_retries: u16,
        sleep_time_millis: u16,
    ) -> Result<(), HomeRadioError> {
        let mut current_retries = 0;
        let duration = Duration::from_millis(sleep_time_millis as u64);
        loop {
            if current_retries > max_retries {
                return Err(HomeRadioError::VLCServerUnhealthy);
            }
            if self.check_health().await? {
                return Ok(());
            }

            sleep(duration).await;
            current_retries += 1;
        }
    }

    pub async fn check_health(&self) -> Result<bool, HomeRadioError> {
        let result = self.get_status().await;
        if result.is_ok() {
            return Ok(true);
        }
        let err = result.err().unwrap();
        match err {
            HomeRadioError::SendRequestError(ref _sre) => Ok(false),
            HomeRadioError::Io(ref io_err) => match io_err.kind() {
                std::io::ErrorKind::ConnectionRefused => Ok(false),
                _ => Err(err),
            },
            _ => Err(err),
        }
    }

    pub async fn get_status(&self) -> Result<VlcStatus, HomeRadioError> {
        let body = self
            .client
            .get(format!("{}/requests/status.json", self.base_url))
            .send()
            .await?
            .body()
            .await?;

        Ok(serde_json::from_str(&String::from_utf8_lossy(&body))?)
    }

    pub async fn stop(&self) -> Result<(), HomeRadioError> {
        self.remote_command("pl_empty", &[]).await?;
        let mut query = HashMap::new();
        query.insert("command", "pl_stop");
        let result = self
            .client
            .get(format!("{}/requests/status.json", self.base_url))
            .query(&query)
            .map_err(|e| HomeRadioError::UrlEncodedError(Box::new(e)))?
            .send()
            .await?;
        dbg!(result);

        Ok(())
    }

    pub async fn set_volume(&self, new_vol: u16) -> Result<(), HomeRadioError> {
        let new_vol = format!("{}", new_vol);
        let mut query = HashMap::new();
        query.insert("command", "volume");
        query.insert("val", &new_vol);
        let request = self
            .client
            .get(format!("{}/requests/status.json", self.base_url))
            .query(&query)
            .map_err(|e| HomeRadioError::UrlEncodedError(Box::new(e)))?;
        dbg!(&request);
        let result = request.send().await?;
        dbg!(result);

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct VlcStatus {
    #[serde(rename = "state")]
    state: String,
}

#[derive(Serialize, Deserialize)]
pub struct Audiofilters {
    #[serde(rename = "filter_0")]
    filter_0: String,
}

#[derive(Serialize, Deserialize)]
pub struct Information {
    #[serde(rename = "chapter")]
    chapter: i64,

    #[serde(rename = "chapters")]
    chapters: Vec<Option<serde_json::Value>>,

    #[serde(rename = "title")]
    title: i64,

    #[serde(rename = "category")]
    category: Category,

    #[serde(rename = "titles")]
    titles: Vec<Option<serde_json::Value>>,
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    #[serde(rename = "meta")]
    meta: Meta,

    #[serde(rename = "Stream 0")]
    stream_0: Option<Stream0>,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    #[serde(rename = "filename")]
    filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct Stream0 {
    #[serde(rename = "Bitrate")]
    bitrate: String,

    #[serde(rename = "Codec")]
    codec: String,

    #[serde(rename = "Bits_pro_Sample")]
    bits_pro_sample: String,

    #[serde(rename = "Abtastrate")]
    abtastrate: String,

    #[serde(rename = "Typ")]
    typ: String,

    #[serde(rename = "KanÃ¤le")]
    kan_le: String,
}

#[derive(Serialize, Deserialize)]
pub struct Videoeffects {
    #[serde(rename = "hue")]
    hue: i64,

    #[serde(rename = "saturation")]
    saturation: i64,

    #[serde(rename = "contrast")]
    contrast: i64,

    #[serde(rename = "brightness")]
    brightness: i64,

    #[serde(rename = "gamma")]
    gamma: i64,
}
