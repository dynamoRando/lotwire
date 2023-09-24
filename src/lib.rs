use config::Config;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, Log};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use rocket::{
    fairing::{Fairing, Info, Kind},
    get,
    http::{Header, Status},
    routes,
    serde::json::Json,
    Request, Response, State,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Debug, Clone)]
pub struct Settings {
    address: String,
    port: u32,
    level: Level,
    num_records: u32,
}

impl Settings {
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
            _ => Level::Error,
        };

        Self {
            address,
            port,
            level,
            num_records,
        }
    }

    pub fn with_values(address: &str, port: u32, level: Level, num_records: u32) -> Self {
        Self {
            address: address.to_string(),
            port,
            level,
            num_records,
        }
    }
}

lazy_static! {
    static ref SERVER: Mutex<LogServer> = Mutex::new(LogServer::default());
}

#[derive(Debug, Clone, Default)]
pub struct LogServer {
    settings: Option<Settings>,
    buffer: Option<Arc<Mutex<AllocRingBuffer<LogItem>>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogItem {
    pub level: String,
    pub module: String,
    pub message: String,
}

impl LogServer {
    pub fn new(dir: &str, filename: &str) -> LogServer {
        let settings = Settings::new(dir, filename);
        Self::init(settings)
    }

    pub fn with_settings(settings: Settings) -> LogServer {
        Self::init(settings)
    }

    fn init(settings: Settings) -> LogServer {
        let buffer = AllocRingBuffer::new(settings.num_records as usize);
        let buffer = Mutex::new(buffer);

        let server = Self {
            settings: Some(settings),
            buffer: Some(buffer.into()),
        };

        *SERVER.lock().unwrap() = server.clone();
        server
    }

    pub fn init_logger(&self) {
        let settings = self.settings.as_ref().unwrap().clone();

        let max_level = match settings.level {
            Level::Error => LevelFilter::Error,
            Level::Warn => LevelFilter::Warn,
            Level::Info => LevelFilter::Info,
            Level::Debug => LevelFilter::Debug,
            Level::Trace => LevelFilter::Trace,
        };

        log::set_max_level(max_level);
        log::set_boxed_logger(Box::new(self.clone())).unwrap();
    }

    pub fn start_server(&self) {
        // println!("Starting server");
        thread::spawn(move || {
            LogServer::start().unwrap();
        });
    }

    #[rocket::main]
    async fn start() -> Result<(), rocket::Error> {
        // println!("Starting server...");
        let server = (*SERVER.lock().unwrap()).clone();
        // println!("Server: {server:?}");
        let settings = server.settings.as_ref().unwrap().clone();

        // println!("Starting server with settings {settings:?}");

        let config = rocket::Config {
            port: settings.port as u16,
            address: settings.address.parse().unwrap(),
            log_level: rocket::config::LogLevel::Off,
            cli_colors: false,
            ..rocket::config::Config::debug_default()
        };

        let _ = rocket::custom(config)
            .attach(CORS)
            .mount("/", routes![index, logs])
            .manage(server)
            .launch()
            .await?;

        Ok(())
    }
}

impl log::Log for LogServer {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.settings.as_ref().unwrap().level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            //println!("Logging...");

            let level = record.level().to_string();
            let module = record.target().to_string();
            let message = record.args().to_string();

            if module.contains("rocket") || module.contains("reqwest") {
                return;
            }

            let item = LogItem {
                level,
                module,
                message,
            };

            self.buffer
                .as_ref()
                .unwrap()
                .as_ref()
                .lock()
                .unwrap()
                .push(item);
        }
    }

    fn flush(&self) {}
}

#[get("/")]
fn index() -> &'static str {
    "Logserver online"
}

#[get("/logs")]
fn logs(server: &State<LogServer>) -> (Status, Json<Vec<LogItem>>) {
    let buffer = server.buffer.as_ref().unwrap().clone();
    let buffer = buffer.lock().unwrap();

    let mut log_items: Vec<LogItem> = Vec::new();

    for item in buffer.iter() {
        log_items.push(item.clone());
    }

    (Status::Ok, Json(log_items))
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS, DELETE",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        response.set_status(Status::Ok)
    }
}
