use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use log::error;
use tokio::{self, runtime::Builder, sync::mpsc};

use crate::errors::HomeRadioError;

use super::{MediaService, MediaServiceFactory};
use awc;

pub enum Message {
    Play(String, u16, mpsc::Sender<Result<(), HomeRadioError>>),
    Stop(mpsc::Sender<Result<(), HomeRadioError>>),
    SetVolume(u16, mpsc::Sender<Result<(), HomeRadioError>>),
}

#[derive(Clone)]
pub struct RemoteMediaService {
    base_url: String,
    host: String,
    port: String,
    client: awc::Client,
}

// pub fn start_media_thread<T: MediaService, F: MediaServiceFactory<T> + Send + 'static>(
//     mut receiver: mpsc::Receiver<Message>,
//     factory: F,
//     initial_volume: u16,
// ) {
//     std::thread::spawn(move || {
//         let rt = Builder::new_current_thread().enable_all().build().unwrap();
//         let srvc = factory.create();
//         let result = srvc.set_volume(initial_volume);
//         if let Err(e) = result {
//             error!("unable to set initial volume: {}", e);
//         }
//         while let Some(msg) = rt.block_on(receiver.recv()) {
//             match msg {
//                 Message::Play(url, volume, result_chan) => {
//                     let result = srvc.start(&url, volume);
//                     rt.block_on(result_chan.send(result)).unwrap();
//                 }
//                 Message::Stop(result_chan) => rt.block_on(result_chan.send(srvc.stop())).unwrap(),
//                 Message::SetVolume(new_vol, result_chan) => {
//                     rt.block_on(result_chan.send(srvc.set_volume(new_vol)))
//                         .unwrap();
//                 }
//             }
//         }
//     });
// }

impl RemoteMediaService {
    pub fn new(host: String, port: String) -> Self {
        let client = awc::Client::builder().finish();
        let base_url = format!("http://{}:{}", host, port);

        RemoteMediaService {
            host,
            port,
            client,
            base_url,
        }
    }

    pub fn new_with_auth(host: String, port: String, user: String, pwd: String) -> Self {
        let client = awc::Client::builder()
            .basic_auth(user, Some(&pwd[..]))
            .finish();
        let base_url = format!("http://{}:{}", host, port);
        RemoteMediaService {
            host,
            port,
            client,
            base_url,
        }
    }

    pub async fn play(&self, url: &str, volume: u16) -> Result<(), HomeRadioError> {
        let mut query = HashMap::new();
        query.insert("command", "in_play");
        query.insert("input", url);
        let (status, response) = tokio::join!(
            self.get_status(),
            self.client
                .get(format!("{}/requests/status.json", self.base_url))
                .query(&query)
                .map_err(|e| HomeRadioError::UrlEncodedError(Box::new(e)))?
                .send()
        );
        let state = status?.state;

        let result = response?;
        dbg!(result);

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

    async fn get_status(&self) -> Result<VlcStatus, HomeRadioError> {
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
