use std::path::Path;

use config::Config;
use log::{Level, Log};


#[derive(Debug, Clone)]
struct Settings {
    address: String,
    port: u32, 
    level: Level,
    num_records: u32
}


impl Settings{
    pub fn new(dir: &str, filename: &str) -> Self {
        let location = Path::new(dir).join(filename.clone());
        let location = location.to_str().unwrap();
        let error_message = "Could not find settings".to_string();

        let settings = Config::builder()
            .add_source(config::File::with_name(location))
            .add_source(config::Environment::with_prefix("APP"))
            .build()
            .expect(&error_message);

        let address = settings.get_string("address").unwrap();
        let port = settings.get_int("port").unwrap() as u32;
        let num_records = settings.get_int("num_messages").unwrap() as u32;
        let str_level = settings.get_string("level").unwrap();
        let str_level = str_level.as_str();

        let level = match str_level {
            "error" => Level::Error,
            "warn" => Level::Warn,
            "info" => Level::Info,
            "debug" => Level::Debug,
            "trace" => Level::Trace,
            _ => Level::Error
        };

        Self {
            address,
            port,
            level,
            num_records
        }
    }
}

pub struct LogServer {
    settings: Settings
}

impl LogServer {
    pub fn new(dir: &str, filename: &str) -> LogServer {
        let settings = Settings::new(dir, filename);
        Self {
            settings
        }
    }
}

impl log::Log for LogServer {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level()  <= self.settings.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            todo!()
        }
    }

    fn flush(&self) {}
}