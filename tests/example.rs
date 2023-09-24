use lotwire::{Settings, LogServer};


#[tokio::test]
async fn test_has_logs(){
    let settings = 
        Settings::with_values("127.0.0.1", 8080, log::Level::Trace, 50);

    let server = LogServer::with_settings(settings);
    server.start_server().await;
}