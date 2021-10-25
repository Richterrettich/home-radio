use log::error;
use tokio::{runtime::Builder, sync::mpsc};

use crate::errors::HomeRadioError;

use super::{MediaService, MediaServiceFactory};

pub enum Message {
    Play(String, u16, mpsc::Sender<Result<(), HomeRadioError>>),
    Stop(mpsc::Sender<Result<(), HomeRadioError>>),
    SetVolume(u16, mpsc::Sender<Result<(), HomeRadioError>>),
}

#[derive(Clone)]
pub struct RemoteMediaService {
    remote: mpsc::Sender<Message>,
}

pub fn start_media_thread<T: MediaService, F: MediaServiceFactory<T> + Send + 'static>(
    mut receiver: mpsc::Receiver<Message>,
    factory: F,
    initial_volume: u16,
) {
    std::thread::spawn(move || {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        let srvc = factory.create();
        let result = srvc.set_volume(initial_volume);
        if let Err(e) = result {
            error!("unable to set initial volume: {}", e);
        }
        while let Some(msg) = rt.block_on(receiver.recv()) {
            match msg {
                Message::Play(url, volume, result_chan) => {
                    let result = srvc.start(&url, volume);
                    rt.block_on(result_chan.send(result)).unwrap();
                }
                Message::Stop(result_chan) => rt.block_on(result_chan.send(srvc.stop())).unwrap(),
                Message::SetVolume(new_vol, result_chan) => {
                    rt.block_on(result_chan.send(srvc.set_volume(new_vol)))
                        .unwrap();
                }
            }
        }
    });
}

impl RemoteMediaService {
    pub fn new(remote: mpsc::Sender<Message>) -> Self {
        RemoteMediaService { remote }
    }

    pub async fn play(&self, url: &str, volume: u16) -> Result<(), HomeRadioError> {
        let (snd, mut recv) = mpsc::channel(1);
        self.remote
            .send(Message::Play(url.to_string(), volume, snd))
            .await
            .map_err(|_| HomeRadioError::ChannelClosed)?;

        recv.recv().await.unwrap()
    }

    pub async fn stop(&self) -> Result<(), HomeRadioError> {
        let (snd, mut recv) = mpsc::channel(1);
        self.remote
            .send(Message::Stop(snd))
            .await
            .map_err(|_| HomeRadioError::ChannelClosed)?;
        recv.recv().await.unwrap()
    }

    pub async fn set_volume(&self, new_vol: u16) -> Result<(), HomeRadioError> {
        let (snd, mut recv) = mpsc::channel(1);
        self.remote
            .send(Message::SetVolume(new_vol, snd))
            .await
            .map_err(|_| HomeRadioError::ChannelClosed)?;
        recv.recv().await.unwrap()
    }
}
