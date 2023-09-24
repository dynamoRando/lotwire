use std::{thread, time::Duration};

use log::{debug, error};
use lotwire::{LogServer, Settings};

#[tokio::test]
async fn test_has_logs() {
    let settings = 
        Settings::with_values("127.0.0.1", 8080, log::Level::Trace, 50);
    let server = LogServer::with_settings(settings);
    server.init_logger();

    debug!("Debug");
    error!("Error");


    thread::spawn(move || {
        server.start_server();
        thread::sleep(Duration::from_secs(1));
    });

    thread::sleep(Duration::from_secs(1));

    let body = reqwest::get("http://127.0.0.1:8080/logs")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    println!("body = {:?}", body);
    assert!(body.len() > 2);
}
