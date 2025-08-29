#[cfg(target_os = "linux")]
use crate::log::*;
use std::process::Command;
use sys_proxy::ProxyConfig;
use url::Url;

#[cfg(target_os = "linux")]
pub fn get_linux_proxy(url: &str) -> Option<String> {
    // Parse the input URL
    let parsed_url = match Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            log_warning(&format!("Failed to parse URL '{}': {}", url, e));
            return None;
        }
    };
    let scheme = parsed_url.scheme();

    // Check GNOME proxy settings
    let gnome_proxy = get_gnome_proxy(&parsed_url);
    if let Some(proxy_url) = gnome_proxy {
        return Some(proxy_url);
    }

    // Check KDE proxy settings
    let kde_proxy = get_kde_proxy(&parsed_url);
    if let Some(proxy_url) = kde_proxy {
        return Some(proxy_url);
    }

    // Fallback to sys-proxy crate
    if let Ok(proxy_config) = ProxyConfig::load() {
        if let Some(proxy_url) = proxy_config.get_proxy_for_url(url) {
            log_info(&format!("Found proxy via sys-proxy: {}", proxy_url));
            return Some(proxy_url);
        }
    }

    log_info(&format!("No applicable proxy found for URL '{}'", url));
    None
}

// Helper function to get GNOME proxy settings
#[cfg(target_os = "linux")]
fn get_gnome_proxy(url: &Url) -> Option<String> {
    // Check GNOME proxy mode
    let mode = Command::new("gsettings")
        .args(["get", "org.gnome.system.proxy", "mode"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().trim_matches('\'').to_string());

    if mode.as_deref() != Some("manual") && mode.as_deref() != Some("auto") {
        log_info("GNOME proxy mode is not set to 'manual' or 'auto'");
        return None;
    }

    if mode.as_deref() == Some("auto") {
        // Get auto-config URL (PAC file)
        if let Some(pac_url) = Command::new("gsettings")
            .args(["get", "org.gnome.system.proxy", "autoconfig-url"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
        {
            log_info(&format!("Found GNOME PAC file: {}", pac_url));
            // Placeholder for PAC file evaluation
            log_warning("PAC file evaluation not implemented yet");
            return None; // TODO: Implement PAC parsing when JS library is chosen
        }
    }

    // Get manual proxy settings
    let scheme = url.scheme();
    let proxy_key = match scheme {
        "https" => "org.gnome.system.proxy.https",
        "http" => "org.gnome.system.proxy.http",
        _ => return None,
    };

    let host = Command::new("gsettings")
        .args(["get", proxy_key, "host"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().trim_matches('\'').to_string())
        .filter(|s| !s.is_empty());

    let port = Command::new("gsettings")
        .args(["get", proxy_key, "port"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|s| s.trim().parse::<u16>().ok());

    if let (Some(host), Some(port)) = (host, port) {
        let proxy_url = format!("{}://{}:{}", scheme, host, port);
        log_info(&format!("Found GNOME {} proxy: {}", scheme, proxy_url));
        return Some(proxy_url);
    }

    None
}

// Helper function to get KDE proxy settings
#[cfg(target_os = "linux")]
fn get_kde_proxy(url: &Url) -> Option<String> {
    // Check KDE proxy settings via kreadconfig5
    let scheme = url.scheme();
    let proxy_type_key = "ProxyType";
    let proxy_type = Command::new("kreadconfig5")
        .args(["--group", "Proxy Settings", "--key", proxy_type_key])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string());

    match proxy_type.as_deref() {
        Some("1") => {
            // Manual proxy
            let proxy_key = match scheme {
                "https" => "httpsProxy",
                "http" => "httpProxy",
                _ => return None,
            };
            let proxy = Command::new("kreadconfig5")
                .args(["--group", "Proxy Settings", "--key", proxy_key])
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            if let Some(proxy_url) = proxy {
                log_info(&format!("Found KDE {} proxy: {}", scheme, proxy_url));
                return Some(proxy_url);
            }
        }
        Some("2") => {
            // PAC file
            if let Some(pac_url) = Command::new("kreadconfig5")
                .args(["--group", "Proxy Settings", "--key", "ProxyConfigScript"])
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
            {
                log_info(&format!("Found KDE PAC file: {}", pac_url));
                // Placeholder for PAC file evaluation
                log_warning("PAC file evaluation not implemented yet");
                return None; // TODO: Implement PAC parsing when JS library is chosen
            }
        }
        _ => {
            log_info("KDE proxy not configured or not supported");
        }
    }
    None
}
