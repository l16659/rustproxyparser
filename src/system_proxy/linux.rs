// src/system_proxy/linux.rs

#[cfg(target_os = "linux")]
use crate::log::*;
use std::process::Command;
use url::Url;

#[cfg(target_os = "linux")]
pub fn get_linux_proxy(url: &str) -> Option<String> {
    let parsed_url = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return None,
    };
    let scheme = parsed_url.scheme();

    // 1. GNOME (most common on Ubuntu/Fedora etc.)
    if let Some(proxy) = get_gnome_proxy(&parsed_url) {
        return Some(proxy);
    }

    // 2. KDE
    if let Some(proxy) = get_kde_proxy(&parsed_url, scheme) {
        return Some(proxy);
    }

    // 3. Fallback: environment variables (some apps set http_proxy etc.)
    if let Some(env_proxy) = std::env::var("http_proxy")
        .ok()
        .or_else(|| std::env::var("HTTP_PROXY").ok())
    {
        if !env_proxy.is_empty() {
            return Some(env_proxy);
        }
    }
    if let Some(env_proxy) = std::env::var("https_proxy")
        .ok()
        .or_else(|| std::env::var("HTTPS_PROXY").ok())
    {
        if !env_proxy.is_empty() {
            return Some(env_proxy);
        }
    }

    None
}

// GNOME proxy (gsettings)
#[cfg(target_os = "linux")]
fn get_gnome_proxy(url: &Url) -> Option<String> {
    let mode_output = Command::new("gsettings")
        .args(["get", "org.gnome.system.proxy", "mode"])
        .output()
        .ok()?;
    let mode = String::from_utf8_lossy(&mode_output.stdout)
        .trim()
        .trim_matches('\'')
        .to_string();

    if mode == "manual" {
        let scheme_key = match url.scheme() {
            "http" => "http",
            "https" => "https",
            _ => return None,
        };

        let host_output = Command::new("gsettings")
            .args([
                "get",
                "org.gnome.system.proxy.http",
                &format!("{}-host", scheme_key),
            ])
            .output()
            .ok()?;
        let host = String::from_utf8_lossy(&host_output.stdout)
            .trim()
            .trim_matches('\'')
            .to_string();

        let port_output = Command::new("gsettings")
            .args([
                "get",
                "org.gnome.system.proxy.http",
                &format!("{}-port", scheme_key),
            ])
            .output()
            .ok()?;
        let port: i32 = String::from_utf8_lossy(&port_output.stdout)
            .trim()
            .parse()
            .unwrap_or(0);

        if !host.is_empty() && port > 0 {
            return Some(format!("http://{}:{}", host, port));
        }
    } else if mode == "auto" {
        let pac_output = Command::new("gsettings")
            .args(["get", "org.gnome.system.proxy", "autoconfig-url"])
            .output()
            .ok()?;
        let pac_url = String::from_utf8_lossy(&pac_output.stdout)
            .trim()
            .trim_matches('\'')
            .to_string();
        if !pac_url.is_empty() {
            log_info!("Found GNOME PAC URL: {}", pac_url);
            return Some(pac_url);
        }
    }

    None
}

// KDE proxy (kreadconfig5 or kreadconfig6)
#[cfg(target_os = "linux")]
fn get_kde_proxy(url: &Url, scheme: &str) -> Option<String> {
    let kread = if Command::new("kreadconfig6").output().is_ok() {
        "kreadconfig6"
    } else {
        "kreadconfig5"
    };

    let proxy_type_output = Command::new(kread)
        .args(["--group", "Proxy Settings", "--key", "ProxyType"])
        .output()
        .ok()?;
    let proxy_type = String::from_utf8_lossy(&proxy_type_output.stdout)
        .trim()
        .to_string();

    if proxy_type == "1" {
        // Manual
        let proxy_key = match scheme {
            "http" => "httpProxy",
            "https" => "httpsProxy",
            _ => return None,
        };

        let proxy_output = Command::new(kread)
            .args(["--group", "Proxy Settings", "--key", proxy_key])
            .output()
            .ok()?;
        let proxy = String::from_utf8_lossy(&proxy_output.stdout)
            .trim()
            .to_string();

        if !proxy.is_empty() {
            return Some(format!("http://{}", proxy));
        }
    } else if proxy_type == "2" {
        // PAC
        let pac_output = Command::new(kread)
            .args(["--group", "Proxy Settings", "--key", "ProxyConfigScript"])
            .output()
            .ok()?;
        let pac_url = String::from_utf8_lossy(&pac_output.stdout)
            .trim()
            .to_string();
        if !pac_url.is_empty() {
            log_info!("Found KDE PAC URL: {}", pac_url);
            return Some(pac_url);
        }
    }

    None
}
