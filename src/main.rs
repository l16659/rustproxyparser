// src/main.rs
use proxyparser::find_proxy_for_url;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let url = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("https://www.google.com");

    match find_proxy_for_url(url) {
        Ok(proxy) => println!("Proxy for {} → {}", url, proxy),
        Err(e) => eprintln!("Error: {}", e),
    }
}
