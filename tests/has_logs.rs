use std::{thread, time::Duration};

use log::{debug, error};
use lotwire::{LogServer, Settings};

/// Tests that the server is recording logs and is available over HTTP. We expect to have a JSON structure of logs returned. An empty array indicates failure.
///
/// Failure indicates potentially multiple problems: There are no logs being recorded, or other issues.
#[tokio::test]
async fn test_has_logs() {
    // -- ARRANGE
    // configure the server with the specified settings and start it
    let settings = Settings::with_values("127.0.0.1", 8080, log::Level::Trace, 50);
    let server = LogServer::with_settings(settings);
    server.init_logger();

    // -- ACT
    // record some example log items
    debug!("Debug");
    error!("Error");

    // -- ARRANGE
    // start the server and wait a second for it to come online
    thread::spawn(move || {
        server.start_server();
        thread::sleep(Duration::from_secs(1));
    });

    // wait for the server to come online
    thread::sleep(Duration::from_secs(1));

    // -- ACT
    // attempt to get the logs from the server over HTTP
    let body = reqwest::get("http://127.0.0.1:8080/logs")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    println!("body = {:?}", body);

    // -- ASSERT
    // ensure the response has a length that is not an empty array
    assert!(body.len() > 2);
}
