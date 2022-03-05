use actix_web::{middleware::Logger, web, App, HttpServer};
use simplelog::{LevelFilter, SimpleLogger};
use tera::Tera;
use toml::de;

use std::fmt;
use std::io;
use std::sync::Arc;

mod config;
mod middleware;
mod routes;

use config::Config;

const LOCALE_DIR: &str = "public/locales";
const TEMPLATE_LOCATION: &str = "frontend/templates/*";

#[actix_web::main]
async fn main() {
    let config = match config::Config::load_config() {
        Ok(c) => Arc::new(c),
        Err(Error::IO(err)) => {
            println!("Could not read config file: {}", err);
            return;
        }
        Err(Error::ConfigParsing(err)) => {
            println!("Could not parse TOML of config file: {}", err);
            return;
        }
    };

    if let Err(err) = SimpleLogger::init(LevelFilter::Info, simplelog::Config::default()) {
        println!("Could not initialize logger: {}", err);
    }

    if let Err(e) = run_webserver(config).await {
        println!("Error while running webserver: {}", e);
    }
}

async fn run_webserver(config: Arc<Config>) -> Result<(), Error> {
    let language_middleware = Arc::new(middleware::locale::LocaleFactory::new(LOCALE_DIR)?);
    let config_data = config.clone();
    let path_map = Arc::new(routes::create_path_map(config.clone()));
    HttpServer::new(move || {
        let tera = Tera::new(TEMPLATE_LOCATION).expect("Could not get templates.");

        App::new()
            .wrap(Logger::default())
            .wrap(language_middleware.clone())
            .app_data(web::Data::new(tera))
            .app_data(web::Data::new(config_data.clone()))
            .app_data(web::Data::new(path_map.clone()))
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
