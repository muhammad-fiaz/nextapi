use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time;

async fn make_api_request(api_url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(api_url).await?;
    Ok(response.text().await?)
}

async fn handle_request(mut stream: TcpStream, data: Arc<Mutex<HashMap<u64, String>>>, tx: mpsc::Sender<String>) {
    let mut buffer = Vec::new();

    // Read the request from the client
    if stream.read_to_end(&mut buffer).is_err() {
        return;
    }

    // Parse the HTTP request (you might want to use a proper HTTP parsing library)
    let request_str = String::from_utf8_lossy(&buffer);
    let request_lines: Vec<&str> = request_str.lines().collect();

    // Assuming a simple HTTP GET request, extract the path from the first line
    let path = if let Some(first_line) = request_lines.first() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            parts[1]
        } else {
            return;
        }
    } else {
        return;
    };

    // Simulate some async work (e.g., waiting for 1 second)
    time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Make the API request based on the extracted path
    let api_url = format!("https://api.example.com{}", path);
    match make_api_request(&api_url).await {
        Ok(api_response) => {
            // Respond to the client with the API response
            let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", api_response);
            if stream.write_all(response.as_bytes()).is_err() {
                return;
            }

            // Store the API response in shared data
            let stream_id = stream.peer_addr().unwrap().port();
            data.lock().unwrap().insert(stream_id as u64, api_response.clone());

            // Notify the response handling thread
            if let Err(err) = tx.send(api_response.clone()).await {
                eprintln!("Error sending response to channel: {}", err);
            }
        }
        Err(err) => {
            // Respond with an error message
            let response = format!("HTTP/1.1 500 Internal Server Error\r\n\r\n{}", err);
            if stream.write_all(response.as_bytes()).is_err() {
                return;
            }
        }
    }

    // Ensure the response is sent immediately
    if stream.flush().is_err() {
        return;
    }
}

pub(crate) async fn start_server(addr: &str, api_url: String, tx: mpsc::Sender<String>) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    println!("Server running on http://{}", addr);

    let data = Arc::new(Mutex::new(HashMap::new()));

    // Spawn a thread to handle incoming requests
    tokio::spawn(async move {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let data = Arc::clone(&data);
                    // Clone the sender directly, no need for Arc<Mutex<...>>
                    let tx_clone = tx.clone();
                    tokio::spawn(handle_request(stream, data, tx_clone));
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    });

    // Simulate the server running indefinitely
    loop {
        time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
