use config::{Config, ConfigError, File};
use serde_derive::Deserialize;
use log::debug;

#[derive(Debug, Deserialize)]
pub struct Web {
    pub listen_addr: String,
    pub static_dir: String,
}
#[derive(Debug, Deserialize)]
pub struct Actions {
    pub sound_dir: String,
    pub video_dir: String,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub web : Web,
}

impl Cfg {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder().
            add_source(File::with_name("config")).
            build()?;
        debug!("config loaded");
        s.try_deserialize()
    }
}
