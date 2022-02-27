
use actix_web::{App, HttpServer};
use toml::de;

use std::fmt;
use std::io;

mod config;
mod routes;

use config::Config;


#[actix_web::main]
async fn main() {
    let config = match config::Config::load_config() {
        Ok(c) => c,
        Err(Error::IO(err)) => {
            println!("Could not read config file: {}", err);
            return;
        },
        Err(Error::ConfigParsing(err)) => {
            println!("Could not parse TOML of config file: {}", err);
            return;
        },
        Err(_) => {
            panic!("Unexpected Error type");
        },
    };

    if let Err(e) = run_webserver(&config).await {
        println!("Error while running webserver: {}", e);
    }
}


async fn run_webserver(config: &Config) -> Result<(), Error> {
    HttpServer::new(|| {
        App::new()
            .service(routes::pages::index)
    })
    .bind(config.local_socket_addr)?
    .run()
    .await?;

    Ok(())
}


#[derive(Debug)]
pub(crate) enum Error {
    IO(io::Error),
    ConfigParsing(de::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            IO(inner) => Some(inner),
            ConfigParsing(inner) => Some(inner),
            _ => None,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            IO(ioerr) => write!(f, "{}", ioerr),
            ConfigParsing(inner) => write!(f, "{}", inner),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}
impl From<de::Error> for Error {
    fn from(err: de::Error) -> Self {
        Error::ConfigParsing(err)
    }
}
