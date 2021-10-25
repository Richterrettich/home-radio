use crate::errors::HomeRadioError;

mod remote_media_service;
mod vlc_media_service;
pub use remote_media_service::*;
pub use vlc_media_service::*;

pub trait MediaServiceFactory<T: MediaService> {
    fn create(&self) -> T;
}

pub trait MediaService {
    fn start(&self, url: &str, volume: u16) -> Result<(), HomeRadioError>;
    fn stop(&self) -> Result<(), HomeRadioError>;
    fn increase_volume(&self, amount: i32) -> Result<(), HomeRadioError>;
    fn decrease_volume(&self, amount: i32) -> Result<(), HomeRadioError>;
    fn get_volume(&self) -> Result<i32, HomeRadioError>;
    fn set_volume(&self, new_vol: u16) -> Result<(), HomeRadioError>;
}


