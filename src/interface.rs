use crate::log::*;
use std::error::Error;

/// The main interface for proxy query.
/// Returns proxy string like "DIRECT", "http://host:port", "socks5://host:port"
pub fn find_proxy_for_url(url: &str) -> Result<String, Box<dyn Error>> {
    // 1. Try environment variable proxy (highest priority)
    if let Some(env_proxy) = crate::env_proxy::get_env_proxy(url) {
        log_info!("Using proxy from environment: {}", env_proxy);
        return Ok(env_proxy);
    }

    // 2. Try system proxy (per platform)
    #[cfg(target_os = "macos")]
    {
        if let Some(sys_proxy) = crate::system_proxy::macos::get_macos_proxy(url) {
            log_info!("Using macOS system proxy: {}", sys_proxy);
            return Ok(sys_proxy);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(sys_proxy) = crate::system_proxy::linux::get_linux_proxy(url) {
            log_info!("Using Linux system proxy: {}", sys_proxy);
            return Ok(sys_proxy);
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(sys_proxy) = crate::system_proxy::windows::get_windows_proxy(url) {
            log_info!("Using Windows system proxy: {}", sys_proxy);
            return Ok(sys_proxy);
        }
    }

    // 3. If PAC file is discovered, download and evaluate (to be implemented)
    // if let Some(pac_proxy) = crate::pac::find_proxy_via_pac(url) {
    //     log_info!("Using PAC proxy: {}", pac_proxy);
    //     return Ok(pac_proxy);
    // }

    // 4. Default: DIRECT
    log_info!("No proxy found, using DIRECT");
    Ok("DIRECT".to_string())
}
