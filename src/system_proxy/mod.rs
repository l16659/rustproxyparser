// src/system_proxy/mod.rs

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

/// 统一获取当前平台的系统代理
/// 返回值：
/// - Some(String)：手动代理（如 "http://127.0.0.1:8080"）或 PAC URL（如 "http://.../proxy.pac"）
/// - None：无代理配置或不支持
pub fn get_system_proxy(url: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        macos::get_macos_proxy(url)
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_linux_proxy(url)
    }
    #[cfg(target_os = "windows")]
    {
        windows::get_windows_proxy(url)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = url; // 未使用参数警告消除
        None
    }
}
