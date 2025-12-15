// src/pac/mod.rs
pub mod downloader;
pub mod engine;

pub use engine::evaluate_pac_for_url;

/// 判断一个字符串是否像是 PAC 脚本 URL
pub fn is_pac_url(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.ends_with(".pac") || lower.contains("proxy.pac") || lower.contains("wpad.dat")
}
