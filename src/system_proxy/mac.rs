#[cfg(target_os = "macos")]
use crate::log::*;
use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::boolean::CFBooleanRef;
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::number::CFNumberRef;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation_sys::base::kCFAllocatorDefault;
use std::ptr;
use system_configuration::dynamic_store::SCDynamicStoreCopyProxies;

#[cfg(target_os = "macos")]
pub fn get_macOS_proxy(url: &str) -> Option<String> {
    // Call SCDynamicStoreCopyProxies to get the proxy configuration dictionary
    let proxies_dict: CFDictionaryRef = unsafe { SCDynamicStoreCopyProxies(ptr::null()) };
    if proxies_dict.is_null() {
        log_warning("Failed to retrieve proxy settings using SCDynamicStoreCopyProxies");
        return None;
    }

    // Ensure the dictionary is released when it goes out of scope
    let _dict_guard = scopeguard::guard(proxies_dict, |dict| {
        unsafe { core_foundation::base::CFRelease(dict as *const _) };
    });

    // Check if Auto Proxy Discovery is enabled
    let auto_proxy_discovery_key = unsafe { kSCPropNetProxiesProxyAutoDiscoveryEnable };
    let auto_proxy_discovery_enabled: CFBooleanRef = unsafe {
        core_foundation::dictionary::CFDictionaryGetValue(proxies_dict, auto_proxy_discovery_key)
    };
    if !auto_proxy_discovery_enabled.is_null() {
        let enabled =
            unsafe { core_foundation::boolean::CFBooleanGetValue(auto_proxy_discovery_enabled) };
        if enabled {
            log_warning("Auto Proxy Discovery is enabled, but not directly supported for proxy URL resolution");
            // Note: Auto Proxy Discovery typically requires resolving via a PAC file or other mechanism
        }
    }

    // Check for PAC (Proxy Auto-Config) proxy
    let pac_key = unsafe { kSCPropNetProxiesProxyAutoConfigURLString };
    let pac_proxy: CFStringRef =
        unsafe { core_foundation::dictionary::CFDictionaryGetValue(proxies_dict, pac_key) };
    if !pac_proxy.is_null() {
        let str_length = unsafe { SafeCFStringGetLength(pac_proxy) };
        if str_length > 0 {
            let pac_url = unsafe { CFString::wrap_under_create_rule(pac_proxy) };
            let pac_url_str = pac_url.to_string();
            log_info(&format!("Found PAC proxy: {}", pac_url_str));
            return Some(pac_url_str);
        }
    }

    // Helper function to extract proxy URL and port
    fn get_proxy_url(
        proxies_dict: CFDictionaryRef,
        proxy_key: CFTypeRef,
        port_key: CFTypeRef,
    ) -> Option<String> {
        let proxy: CFStringRef =
            unsafe { core_foundation::dictionary::CFDictionaryGetValue(proxies_dict, proxy_key) };
        if !proxy.is_null() {
            let str_length = unsafe { SafeCFStringGetLength(proxy) };
            if str_length > 0 {
                let proxy_str = unsafe { CFString::wrap_under_create_rule(proxy) }.to_string();
                let port: CFNumberRef = unsafe {
                    core_foundation::dictionary::CFDictionaryGetValue(proxies_dict, port_key)
                };
                let port_str = if !port.is_null() {
                    let mut port_num: i32 = 0;
                    let success = unsafe {
                        core_foundation::number::CFNumberGetValue(
                            port,
                            core_foundation::number::kCFNumberSInt32Type,
                            &mut port_num as *mut _ as *mut _,
                        )
                    };
                    if success {
                        format!(":{}", port_num)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                return Some(format!("{}:{}", proxy_str, port_str));
            }
        }
        None
    }

    // Check for HTTPS proxy
    let https_proxy_key = unsafe { kSCPropNetProxiesHTTPSProxy };
    let https_port_key = unsafe { kSCPropNetProxiesHTTPSPort };
    if let Some(https_proxy) = get_proxy_url(proxies_dict, https_proxy_key, https_port_key) {
        log_info(&format!("Found HTTPS proxy: {}", https_proxy));
        return Some(format!("https://{}", https_proxy));
    }

    // Check for HTTP proxy
    let http_proxy_key = unsafe { kSCPropNetProxiesHTTPProxy };
    let http_port_key = unsafe { kSCPropNetProxiesHTTPPort };
    if let Some(http_proxy) = get_proxy_url(proxies_dict, http_proxy_key, http_port_key) {
        log_info(&format!("Found HTTP proxy: {}", http_proxy));
        return Some(format!("http://{}", http_proxy));
    }

    // Check for SOCKS proxy
    let socks_proxy_key = unsafe { kSCPropNetProxiesSOCKSProxy };
    let socks_port_key = unused {
        kSCPropNetProxiesSOCKSPort,
    };
    if let Some(socks_proxy) = get_proxy_url(proxies_dict, socks_proxy_key, socks_port_key) {
        log_info(&format!("Found SOCKS proxy: {}", socks_proxy));
        return Some(format!("socks5://{}", socks_proxy));
    }

    // No proxy found
    log_info("No applicable proxy found for the given URL");
    None
}
