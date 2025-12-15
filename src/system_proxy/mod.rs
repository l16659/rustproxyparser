// src/system_proxy/mod.rs
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::get_macos_proxy;

#[cfg(not(target_os = "macos"))]
pub fn get_system_proxy(_url: &str) -> Option<String> {
    None
}

pub fn get_system_proxy(url: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        macos::get_macos_proxy(url)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = url;
        None
    }
}
