use std::sync::mpsc;

use log::debug;
use vlc::{EventType, Instance, Media, MediaPlayer, MediaPlayerAudioEx};

use crate::errors::HomeRadioError;

use super::{MediaService, MediaServiceFactory};

pub struct VLCMediaServiceFactory;

impl MediaServiceFactory<VLCMediaService> for VLCMediaServiceFactory {
    fn create(&self) -> VLCMediaService {
        let instance = Instance::new().unwrap();
        let player = MediaPlayer::new(&instance).unwrap();

        VLCMediaService { instance, player }
    }
}

pub struct VLCMediaService {
    instance: Instance,
    player: MediaPlayer,
}

impl MediaService for VLCMediaService {
    fn start(&self, url: &str, volume: u16) -> Result<(), HomeRadioError> {
        let url = url.as_ref();
        let md = Media::new_location(&self.instance, url);
        let md = md.ok_or_else(|| HomeRadioError::NoMedia(url.to_string()))?;
        let event_manager = self.player.event_manager();
        let (tx, rx) = mpsc::sync_channel::<()>(0);
        let ptr = event_manager
            .attach(EventType::MediaPlayerTimeChanged, move |e, _o| {
                debug!("got media player playing event {:?}", &e);
                let _ = tx.try_send(());
            })
            .map_err(|_| HomeRadioError::Play)?;

        self.player.set_media(&md);

        self.player.play().map_err(|_e| HomeRadioError::Play)?;
        debug!("waiting for playing state");
        rx.recv().unwrap();
        event_manager
            .detach(EventType::MediaPlayerTimeChanged, ptr)
            .map_err(|_| HomeRadioError::Play)?;

        debug!("setting volume");
        self.player
            .set_volume(volume as i32)
            .map_err(|_e| HomeRadioError::VolumeChange(volume as i32))
    }

    fn stop(&self) -> Result<(), HomeRadioError> {
        self.player.stop();
        Ok(())
    }

    fn increase_volume(&self, amount: i32) -> Result<(), HomeRadioError> {
        let new_vol = self.player.get_volume() + amount;
        self.player
            .set_volume(new_vol)
            .map_err(|_e| HomeRadioError::VolumeChange(new_vol))
    }

    fn decrease_volume(&self, amount: i32) -> Result<(), HomeRadioError> {
        let new_vol = self.player.get_volume() - amount;
        self.player
            .set_volume(new_vol)
            .map_err(|_e| HomeRadioError::VolumeChange(new_vol))
    }

    fn get_volume(&self) -> Result<i32, HomeRadioError> {
        Ok(self.player.get_volume())
    }

    fn set_volume(&self, new_vol: u16) -> Result<(), HomeRadioError> {
        if self.player.is_playing() {
            self.player
                .set_volume(new_vol as i32)
                .map_err(|_e| HomeRadioError::VolumeChange(new_vol as i32))
        } else {
            Ok(())
        }
    }
}
