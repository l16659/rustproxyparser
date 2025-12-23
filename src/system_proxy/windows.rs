// src/system_proxy/windows.rs

#[cfg(target_os = "windows")]
use crate::{log_info, log_warn};
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winhttp::{
    WinHttpCloseHandle, WinHttpGetIEProxyConfigForCurrentUser, WinHttpGetProxyForUrl, WinHttpOpen,
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_AUTOPROXY_OPTIONS, WINHTTP_AUTO_DETECT_TYPE_DHCP,
    WINHTTP_AUTO_DETECT_TYPE_DNS_A, WINHTTP_CURRENT_USER_IE_PROXY_CONFIG, WINHTTP_PROXY_INFO,
};

#[cfg(target_os = "windows")]
pub fn get_windows_proxy(url: &str) -> Option<String> {
    let url_wide: Vec<u16> = OsStr::new(url)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // 第一步：读取 IE 代理配置（手动代理或 PAC URL）
    let mut ie_config = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG {
        fAutoDetect: 0,
        lpszAutoConfigUrl: ptr::null_mut(),
        lpszProxy: ptr::null_mut(),
        lpszProxyBypass: ptr::null_mut(),
    };

    let success = unsafe { WinHttpGetIEProxyConfigForCurrentUser(&mut ie_config) };
    if success == 0 {
        log_warn!("WinHttpGetIEProxyConfigForCurrentUser failed: {}", unsafe {
            GetLastError()
        });
        return None;
    }

    let _guard = scopeguard::guard((), |_| unsafe {
        if !ie_config.lpszAutoConfigUrl.is_null() {
            winapi::um::winbase::GlobalFree(ie_config.lpszAutoConfigUrl as _);
        }
        if !ie_config.lpszProxy.is_null() {
            winapi::um::winbase::GlobalFree(ie_config.lpszProxy as _);
        }
        if !ie_config.lpszProxyBypass.is_null() {
            winapi::um::winbase::GlobalFree(ie_config.lpszProxyBypass as _);
        }
    });

    // 如果有 PAC URL，直接返回
    if !ie_config.lpszAutoConfigUrl.is_null() {
        let pac_url = wide_ptr_to_string(ie_config.lpszAutoConfigUrl);
        if !pac_url.is_empty() {
            log_info!("Found PAC URL from IE config: {}", pac_url);
            return Some(pac_url);
        }
    }

    // 如果有手动代理
    if !ie_config.lpszProxy.is_null() {
        let proxy = wide_ptr_to_string(ie_config.lpszProxy);
        if !proxy.is_empty() {
            log_info!("Found manual proxy from IE config: {}", proxy);
            return Some(format!("http://{}", proxy));
        }
    }

    // 如果开启自动检测，尝试 WPAD
    if ie_config.fAutoDetect != 0 {
        let session = unsafe {
            WinHttpOpen(
                ptr::null(),
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                ptr::null(),
                ptr::null(),
                0,
            )
        };
        if session.is_null() {
            log_warn!("WinHttpOpen failed for WPAD");
            return None;
        }

        let mut auto_options = WINHTTP_AUTOPROXY_OPTIONS {
            dwFlags: WINHTTP_AUTO_DETECT_TYPE_DHCP | WINHTTP_AUTO_DETECT_TYPE_DNS_A,
            dwAutoDetectFlags: 0,
            lpszAutoConfigUrl: ptr::null_mut(),
            lpvReserved: ptr::null_mut(),
            dwReserved: 0,
            fAutoLogonIfChallenged: 1,
        };

        let mut proxy_info = WINHTTP_PROXY_INFO {
            dwAccessType: 0,
            lpszProxy: ptr::null_mut(),
            lpszProxyBypass: ptr::null_mut(),
        };

        let result = unsafe {
            WinHttpGetProxyForUrl(
                session,
                url_wide.as_ptr(),
                &mut auto_options,
                &mut proxy_info,
            )
        };

        unsafe { WinHttpCloseHandle(session) };

        if result != 0 {
            let _proxy_guard = scopeguard::guard((), |_| unsafe {
                if !proxy_info.lpszProxy.is_null() {
                    winapi::um::winbase::GlobalFree(proxy_info.lpszProxy as _);
                }
                if !proxy_info.lpszProxyBypass.is_null() {
                    winapi::um::winbase::GlobalFree(proxy_info.lpszProxyBypass as _);
                }
            });

            if !proxy_info.lpszProxy.is_null() {
                let proxy = wide_ptr_to_string(proxy_info.lpszProxy);
                if !proxy.is_empty() {
                    log_info!("Found proxy via WPAD: {}", proxy);
                    return Some(format!("http://{}", proxy));
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn wide_ptr_to_string(ptr: *mut u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let len = unsafe { (0..).take_while(|&i| *ptr.offset(i) != 0).count() };
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    OsString::from_wide(slice).to_string_lossy().into_owned()
}
