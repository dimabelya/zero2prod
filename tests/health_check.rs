use std::{fmt::format, mem::zeroed, net::TcpListener};
use zero2prod::run;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
	.expect("Failed to execute a request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length()); 
}

// Launch our application in the background
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");

    // Retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    // Return the application address to the caller
    format!("http://127.0.0.1:{}", port)
}
