// src/client.rs

use reqwest;

pub async fn send_request_to_server(server_url: &str) -> Result<String, reqwest::Error> {
    // You can customize this request according to your needs
    let response = reqwest::get(server_url).await?;
    Ok(response.text().await?)
}
