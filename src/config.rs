use clap::Parser;
use serde::Deserialize;

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::Error;

const DEFAULT_CONFIG_PATH: &str = r"/etc/rust-send/config.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default = "default_servername")]
    pub servername: String,
    #[serde(default = "default_local_socket_addr")]
    pub local_socket_addr: SocketAddr,
}
impl Config {
    pub fn load_config() -> Result<Self, Error> {
        let args = Args::parse();

        let config = if let Some(config_path) = args.config_location {
            Config::try_from(File::open(config_path)?)?
        } else {
            match File::open(&DEFAULT_CONFIG_PATH) {
                Ok(file) => Config::try_from(file)?,
                Err(err) if err.kind() == io::ErrorKind::NotFound => Config::default(),
                Err(err) => return Err(err.into()),
            }
        };

        Ok(config)
    }
}
impl Default for Config {
    fn default() -> Self {
        Config {
            servername: default_servername(),
            local_socket_addr: default_local_socket_addr(),
        }
    }
}
impl TryFrom<File> for Config {
    type Error = Error;

    fn try_from(mut file: File) -> Result<Self, Error> {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(toml::from_slice(&buf)?)
    }
}

#[derive(Parser)]
#[clap(name = "rust-send")]
#[clap(author = "Simeon Ricking <simeon.ricking@simusense.eu>")]
#[clap(
    about = "File-sharing server",
    long_about = "A file sharing server based on Firefox-Send"
)]
struct Args {
    /// Path to the configuration file
    #[clap(name = "config")]
    #[clap(short, long)]
    #[clap(help = "Path to the configuration file")]
    config_location: Option<OsString>,
}

fn default_servername() -> String {
    String::from("example.com")
}
fn default_local_socket_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
}
