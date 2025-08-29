#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Unified system proxy query for current platform
pub fn get_system_proxy(url: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        macos::get_macOS_proxy(url)
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
        None
    }
}
