use std::env;
use crate::log::*;

/// Return proxy string from environment variables if present, else None.
/// - Honors HTTP_PROXY, HTTPS_PROXY, NO_PROXY, etc.
/// - Highest priority.
pub fn get_env_proxy(url: &str) -> Option<String> {
    let url_lc = url.to_lowercase();
    let env_vars = [
        "http_proxy",
        "https_proxy",
        "ftp_proxy",
        "socks_proxy",
        "no_proxy",
    ];
    // If any NO_PROXY matches, return "DIRECT"
    if let Ok(no_proxy) = env::var("no_proxy").or(env::var("NO_PROXY")) {
        if is_url_in_no_proxy(url, &no_proxy) {
            log_info!("URL {} is in NO_PROXY list", url);
            return Some("DIRECT".to_string());
        }
    }
    // Find matching proxy
    for var in &env_vars {
        if let Ok(proxy) = env::var(var).or(env::var(&var.to_uppercase())) {
            if !proxy.is_empty() {
                // Only match http/https/ftp/socks if url scheme matches
                if url_lc.starts_with(&var[..var.find('_').unwrap_or(0)]) || var == &"no_proxy" {
                    log_info!("Proxy {} found for scheme {}", proxy, var);
                    return Some(proxy);
                }
            }
        }
    }
    None
}

/// Simple no_proxy filter
fn is_url_in_no_proxy(url: &str, no_proxy: &str) -> bool {
    // Very basic match: check if host ends with any entry in no_proxy
    let host = match url::Url::parse(url).ok().and_then(|u| u.host_str().map(|s| s.to_string())) {
        Some(h) => h,
        None => return false,
    };
    for entry in no_proxy.split(',') {
        let entry_trim = entry.trim();
        if entry_trim.is_empty() { continue; }
        if host.ends_with(entry_trim) {
            return true;
        }
    }
    false
}
