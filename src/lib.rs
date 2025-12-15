// src/lib.rs
pub mod env_proxy;
pub mod log;
pub mod pac;
pub mod system_proxy;

use log::*;
use std::error::Error;

pub fn find_proxy_for_url(url: &str) -> Result<String, Box<dyn Error>> {
    // 1. 环境变量最高优先级
    if let Some(proxy) = env_proxy::get_env_proxy(url) {
        log_info!("Using environment proxy: {}", proxy);
        return Ok(proxy);
    }

    // 2. 系统代理（macOS）
    if let Some(system_result) = system_proxy::get_system_proxy(url) {
        if pac::is_pac_url(&system_result) {
            log_info!("Detected PAC configuration: {}", system_result);
            match pac::evaluate_pac_for_url(&system_result, url) {
                Ok(proxy) => {
                    log_info!("PAC resolved proxy: {}", proxy);
                    return Ok(proxy);
                }
                Err(e) => {
                    log_warn!("PAC evaluation failed: {}. Falling back to DIRECT", e);
                }
            }
        } else {
            log_info!("Using system manual proxy: {}", system_result);
            return Ok(system_result);
        }
    }

    log_info!("No proxy found, using DIRECT");
    Ok("DIRECT".to_string())
}
