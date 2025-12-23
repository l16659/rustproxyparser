// src/pac/downloader.rs
use crate::{log_error, log_info};
use reqwest::blocking::Client;
use std::time::Duration;

pub fn download_pac(pac_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    log_info!("Downloading PAC script from: {}", pac_url);

    let client = Client::builder().timeout(Duration::from_secs(15)).build()?;

    let response = client.get(pac_url).send()?;

    if !response.status().is_success() {
        log_error!("PAC download failed: HTTP {}", response.status());
        return Err(format!("HTTP {}", response.status()).into());
    }

    let text = response.text()?;
    log_info!("PAC script downloaded successfully ({} bytes)", text.len());
    Ok(text)
}
