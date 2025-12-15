// src/system_proxy/macos.rs

#[cfg(target_os = "macos")]
use crate::{log_info, log_warn};
#[cfg(target_os = "macos")]
use core_foundation::base::{CFRelease, TCFType};
use core_foundation::dictionary::{CFDictionaryGetValue, CFDictionaryRef};
use core_foundation::number::{kCFNumberSInt32Type, CFNumberGetValue, CFNumberRef};
use core_foundation::string::{CFString, CFStringRef};
use scopeguard::defer;
use std::ptr;

// 使用 sys crate 的 FFI 接口（全局依赖，确保不 private）
use system_configuration_sys::dynamic_store_copy_specific::SCDynamicStoreCopyProxies;

const kSCPropNetProxiesProxyAutoConfigURLString: &str = "ProxyAutoConfigURLString";
const kSCPropNetProxiesHTTPSProxy: &str = "HTTPSProxy";
const kSCPropNetProxiesHTTPSPort: &str = "HTTPSPort";
const kSCPropNetProxiesHTTPProxy: &str = "HTTPProxy";
const kSCPropNetProxiesHTTPPort: &str = "HTTPPort";
const kSCPropNetProxiesSOCKSProxy: &str = "SOCKSProxy";
const kSCPropNetProxiesSOCKSPort: &str = "SOCKSPort";

#[cfg(target_os = "macos")]
pub fn get_macos_proxy(_url: &str) -> Option<String> {
    let pac_key = CFString::new(kSCPropNetProxiesProxyAutoConfigURLString);
    let https_proxy_key = CFString::new(kSCPropNetProxiesHTTPSProxy);
    let https_port_key = CFString::new(kSCPropNetProxiesHTTPSPort);
    let http_proxy_key = CFString::new(kSCPropNetProxiesHTTPProxy);
    let http_port_key = CFString::new(kSCPropNetProxiesHTTPPort);
    let socks_proxy_key = CFString::new(kSCPropNetProxiesSOCKSProxy);
    let socks_port_key = CFString::new(kSCPropNetProxiesSOCKSPort);

    let proxies_dict: CFDictionaryRef = unsafe { SCDynamicStoreCopyProxies(ptr::null()) };
    if proxies_dict.is_null() {
        log_warn!("Failed to retrieve macOS proxy settings");
        return None;
    }

    defer! { unsafe { CFRelease(proxies_dict as *const _) }; }

    // 检查 PAC URL
    let pac_val: *const std::os::raw::c_void =
        unsafe { CFDictionaryGetValue(proxies_dict, pac_key.as_concrete_TypeRef() as *const _) };
    if !pac_val.is_null() {
        let pac_url = unsafe { CFString::wrap_under_get_rule(pac_val as CFStringRef) };
        let pac_str = pac_url.to_string();
        if !pac_str.is_empty() {
            log_info!("Found PAC URL: {}", pac_str);
            return Some(pac_str);
        }
    }

    let get_host_port = |host_key: &CFString, port_key: &CFString| -> Option<String> {
        let host_val: *const std::os::raw::c_void = unsafe {
            CFDictionaryGetValue(proxies_dict, host_key.as_concrete_TypeRef() as *const _)
        };
        if host_val.is_null() {
            return None;
        }
        let host = unsafe { CFString::wrap_under_get_rule(host_val as CFStringRef) }.to_string();

        let port_val: *const std::os::raw::c_void = unsafe {
            CFDictionaryGetValue(proxies_dict, port_key.as_concrete_TypeRef() as *const _)
        };
        let port = if !port_val.is_null() {
            let mut num: i32 = 0;
            let ok = unsafe {
                CFNumberGetValue(
                    port_val as CFNumberRef,
                    kCFNumberSInt32Type,
                    &mut num as *mut i32 as *mut std::os::raw::c_void,
                )
            };
            if ok && num > 0 {
                format!(":{}", num)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        Some(format!("{}{}", host, port))
    };

    if let Some(host_port) = get_host_port(&https_proxy_key, &https_port_key) {
        log_info!("Found HTTPS proxy: https://{}", host_port);
        return Some(format!("https://{}", host_port));
    }
    if let Some(host_port) = get_host_port(&http_proxy_key, &http_port_key) {
        log_info!("Found HTTP proxy: http://{}", host_port);
        return Some(format!("http://{}", host_port));
    }
    if let Some(host_port) = get_host_port(&socks_proxy_key, &socks_port_key) {
        log_info!("Found SOCKS proxy: socks5://{}", host_port);
        return Some(format!("socks5://{}", host_port));
    }

    None
}
