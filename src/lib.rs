// src/main.rs

mod server;
mod client;

use tokio::sync::mpsc;
use server::start_server;
use client::send_request_to_server;

#[tokio::main]
async fn main() {
    // Channel for sending API responses to the server
    let (tx, mut rx) = mpsc::channel(1);

    // Start the server
    let server_handle = tokio::spawn(start_server("127.0.0.1:8080", "https://www.example.com/api".to_string(), tx));

    // Wait for the server to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Send a sample request to the server
    match send_request_to_server("http://127.0.0.1:8080").await {
        Ok(response) => {
            println!("Received API response in main: {}", response);
            // Handle the API response as needed
        }
        Err(e) => {
            eprintln!("Error sending request: {}", e);
        }
    }

    // Handle API responses in the main thread
    while let Some(api_response) = rx.recv().await {
        println!("Received API response in main: {}", api_response);
        // Handle the API response as needed
    }

    // Wait for the server to finish
    if let Err(e) = server_handle.await {
        eprintln!("Server error: {}", e);
    }
}
