#[cfg(target_os = "windows")]
use crate::log::*;
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winhttp::{
    WinHttpGetIEProxyConfigForCurrentUser, WINHTTP_AUTOPROXY_OPTIONS,
    WINHTTP_CURRENT_USER_IE_PROXY_CONFIG, WINHTTP_PROXY_INFO,
};
use winapi::um::winnt::LPWSTR;

#[cfg(target_os = "windows")]
pub fn get_windows_proxy(url: &str) -> Option<String> {
    // Convert URL to wide string for WinHTTP
    let url_wide: Vec<u16> = OsStr::new(url)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Retrieve IE proxy configuration for the current user
    let mut proxy_config = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG {
        fAutoDetect: 0,
        lpszAutoConfigUrl: ptr::null_mut(),
        lpszProxy: ptr::null_mut(),
        lpszProxyBypass: ptr::null_mut(),
    };

    let result = unsafe { WinHttpGetIEProxyConfigForCurrentUser(&mut proxy_config) };
    if result == 0 {
        let error = unsafe { GetLastError() };
        log_warning(&format!(
            "Failed to retrieve IE proxy config, error code: {}",
            error
        ));
        return None;
    }

    // Scope guard to free allocated strings
    let _guard = scopeguard::guard((), |_| unsafe {
        if !proxy_config.lpszAutoConfigUrl.is_null() {
            winapi::um::winbase::GlobalFree(proxy_config.lpszAutoConfigUrl as _);
        }
        if !proxy_config.lpszProxy.is_null() {
            winapi::um::winbase::GlobalFree(proxy_config.lpszProxy as _);
        }
        if !proxy_config.lpszProxyBypass.is_null() {
            winapi::um::winbase::GlobalFree(proxy_config.lpszProxyBypass as _);
        }
    });

    // Check for manual proxy
    if !proxy_config.lpszProxy.is_null() {
        let proxy_str = unsafe {
            let len = (0..)
                .take_while(|&i| *proxy_config.lpszProxy.offset(i) != 0)
                .count();
            let slice = std::slice::from_raw_parts(proxy_config.lpszProxy, len);
            OsString::from_wide(slice).to_string_lossy().into_owned()
        };

        // Parse proxy string (e.g., "http=proxy:port;https=proxy:port")
        let proxy = parse_proxy_string(&proxy_str, url);
        if let Some(proxy_url) = proxy {
            log_info(&format!("Found manual proxy: {}", proxy_url));
            return Some(proxy_url);
        }
    }

    // Check for auto-config (PAC) file
    if !proxy_config.lpszAutoConfigUrl.is_null() {
        let pac_url = unsafe {
            let len = (0..)
                .take_while(|&i| *proxy_config.lpszAutoConfigUrl.offset(i) != 0)
                .count();
            let slice = std::slice::from_raw_parts(proxy_config.lpszAutoConfigUrl, len);
            OsString::from_wide(slice).to_string_lossy().into_owned()
        };
        log_info(&format!("Found PAC file: {}", pac_url));

        // Attempt to resolve proxy using PAC file
        if let Some(proxy_url) = resolve_pac_proxy(&pac_url, url) {
            log_info(&format!("Resolved proxy from PAC: {}", proxy_url));
            return Some(proxy_url);
        } else {
            log_warning("Failed to resolve proxy from PAC file");
        }
    }

    // Check for auto-detect proxy
    if proxy_config.fAutoDetect != 0 {
        log_info("Auto-detect proxy enabled, attempting WPAD");
        if let Some(proxy_url) = resolve_auto_proxy(url, &url_wide) {
            log_info(&format!("Resolved proxy via WPAD: {}", proxy_url));
            return Some(proxy_url);
        } else {
            log_warning("Failed to resolve proxy via WPAD");
        }
    }

    // Fallback to system-wide WinHTTP proxy settings
    let mut winhttp_proxy_info = WINHTTP_PROXY_INFO {
        dwAccessType: 0,
        lpszProxy: ptr::null_mut(),
        lpszProxyBypass: ptr::null_mut(),
    };
    let result = unsafe {
        winapi::um::winhttp::WinHttpGetDefaultProxyConfiguration(&mut winhttp_proxy_info)
    };
    if result == ERROR_SUCCESS && !winhttp_proxy_info.lpszProxy.is_null() {
        let proxy_str = unsafe {
            let len = (0..)
                .take_while(|&i| *winhttp_proxy_info.lpszProxy.offset(i) != 0)
                .count();
            let slice = std::slice::from_raw_parts(winhttp_proxy_info.lpszProxy, len);
            OsString::from_wide(slice).to_string_lossy().into_owned()
        };
        unsafe {
            if !winhttp_proxy_info.lpszProxy.is_null() {
                winapi::um::winbase::GlobalFree(winhttp_proxy_info.lpszProxy as _);
            }
            if !winhttp_proxy_info.lpszProxyBypass.is_null() {
                winapi::um::winbase::GlobalFree(winhttp_proxy_info.lpszProxyBypass as _);
            }
        }
        let proxy = parse_proxy_string(&proxy_str, url);
        if let Some(proxy_url) = proxy {
            log_info(&format!("Found system-wide WinHTTP proxy: {}", proxy_url));
            return Some(proxy_url);
        }
    }

    log_info("No applicable proxy found for the given URL");
    None
}

// Helper function to parse proxy string (e.g., "http=proxy:port;https=proxy:port")
fn parse_proxy_string(proxy_str: &str, url: &str) -> Option<String> {
    let scheme = if url.starts_with("https://") {
        "https"
    } else if url.starts_with("http://") {
        "http"
    } else {
        return None;
    };

    for part in proxy_str.split(';') {
        if part.starts_with(&format!("{}=", scheme)) {
            let proxy = part.split('=').nth(1)?;
            return Some(format!("{}://{}", scheme, proxy));
        } else if !part.contains('=') && !part.is_empty() {
            // Assume single proxy for all protocols
            return Some(format!("{}://{}", scheme, part));
        }
    }
    None
}

// Helper function to resolve proxy from PAC file (simplified)
fn resolve_pac_proxy(_pac_url: &str, _url: &str) -> Option<String> {
    // Note: Full PAC file evaluation requires a JavaScript engine (e.g., v8 crate)
    // For simplicity, assume PAC file returns a direct proxy or NONE
    // In a real implementation, download and evaluate the PAC file using a JS engine
    log_warning("PAC file evaluation not implemented; requires JavaScript engine");
    None
}

// Helper function to resolve proxy via WPAD
fn resolve_auto_proxy(url: &str, url_wide: &[u16]) -> Option<String> {
    let mut session = ptr::null_mut();
    let session_handle = unsafe {
        winapi::um::winhttp::WinHttpOpen(
            ptr::null(),
            winapi::um::winhttp::WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            ptr::null(),
            ptr::null(),
            0,
        )
    };
    if session_handle.is_null() {
        log_warning("Failed to open WinHTTP session for WPAD");
        return None;
    }
    session = session_handle;

    let _guard = scopeguard::guard(session, |s| {
        if !s.is_null() {
            unsafe { winapi::um::winhttp::WinHttpCloseHandle(s as _) };
        }
    });

    let mut auto_proxy_options = WINHTTP_AUTOPROXY_OPTIONS {
        dwFlags: winapi::um::winhttp::WINHTTP_AUTOPROXY_AUTO_DETECT,
        dwAutoDetectFlags: winapi::um::winhttp::WINHTTP_AUTO_DETECT_TYPE_DHCP
            | winapi::um::winhttp::WINHTTP_AUTO_DETECT_TYPE_DNS_A,
        fAutoLogonIfChallenged: 1,
        lpszAutoConfigUrl: ptr::null_mut(),
        lpvReserved: ptr::null_mut(),
        dwReserved: 0,
    };

    let mut proxy_info = WINHTTP_PROXY_INFO {
        dwAccessType: 0,
        lpszProxy: ptr::null_mut(),
        lpszProxyBypass: ptr::null_mut(),
    };

    let result = unsafe {
        winapi::um::winhttp::WinHttpGetProxyForUrl(
            session,
            url_wide.as_ptr(),
            &mut auto_proxy_options,
            &mut proxy_info,
        )
    };
    if result == 0 {
        let error = unsafe { GetLastError() };
        log_warning(&format!(
            "Failed to resolve proxy via WPAD, error code: {}",
            error
        ));
        return None;
    }

    let _guard = scopeguard::guard((), |_| unsafe {
        if !proxy_info.lpszProxy.is_null() {
            winapi::um::winbase::GlobalFree(proxy_info.lpszProxy as _);
        }
        if !proxy_info.lpszProxyBypass.is_null() {
            winapi::um::winbase::GlobalFree(proxy_info.lpszProxyBypass as _);
        }
    });

    if !proxy_info.lpszProxy.is_null() {
        let proxy_str = unsafe {
            let len = (0..)
                .take_while(|&i| *proxy_info.lpszProxy.offset(i) != 0)
                .count();
            let slice = std::slice::from_raw_parts(proxy_info.lpszProxy, len);
            OsString::from_wide(slice).to_string_lossy().into_owned()
        };
        let scheme = if url.starts_with("https://") {
            "https"
        } else {
            "http"
        };
        Some(format!("{}://{}", scheme, proxy_str))
    } else {
        None
    }
}
