#[cfg(target_os = "macos")]
use crate::log::*;
use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::number::CFNumberRef;
use core_foundation::string::{CFString, CFStringRef};
use std::ptr;
use system_configuration::dynamic_store::SCDynamicStoreCopyProxies;

// 添加缺失的常量定义
const kSCPropNetProxiesProxyAutoDiscoveryEnable: &str = "ProxyAutoDiscoveryEnable";
const kSCPropNetProxiesProxyAutoConfigURLString: &str = "ProxyAutoConfigURLString";
const kSCPropNetProxiesHTTPSProxy: &str = "HTTPSProxy";
const kSCPropNetProxiesHTTPSPort: &str = "HTTPSPort";
const kSCPropNetProxiesHTTPProxy: &str = "HTTPProxy";
const kSCPropNetProxiesHTTPPort: &str = "HTTPPort";
const kSCPropNetProxiesSOCKSProxy: &str = "SOCKSProxy";
const kSCPropNetProxiesSOCKSPort: &str = "SOCKSPort";

#[cfg(target_os = "macos")]
pub fn get_macos_proxy(url: &str) -> Option<String> {
    // 将字符串常量转换为CFString
    let auto_proxy_discovery_key = CFString::new(kSCPropNetProxiesProxyAutoDiscoveryEnable);
    let pac_key = CFString::new(kSCPropNetProxiesProxyAutoConfigURLString);
    let https_proxy_key = CFString::new(kSCPropNetProxiesHTTPSProxy);
    let https_port_key = CFString::new(kSCPropNetProxiesHTTPSPort);
    let http_proxy_key = CFString::new(kSCPropNetProxiesHTTPProxy);
    let http_port_key = CFString::new(kSCPropNetProxiesHTTPPort);
    let socks_proxy_key = CFString::new(kSCPropNetProxiesSOCKSProxy);
    let socks_port_key = CFString::new(kSCPropNetProxiesSOCKSPort);

    // Call SCDynamicStoreCopyProxies to get the proxy configuration dictionary
    let proxies_dict: CFDictionaryRef = unsafe { SCDynamicStoreCopyProxies(ptr::null()) };
    if proxies_dict.is_null() {
        log_warning!("Failed to retrieve proxy settings using SCDynamicStoreCopyProxies");
        return None;
    }

    // Ensure the dictionary is released when it goes out of scope
    let _dict_guard = scopeguard::guard(proxies_dict, |dict| {
        unsafe { core_foundation::base::CFRelease(dict as *const _) };
    });

    // Check if Auto Proxy Discovery is enabled
    let auto_proxy_discovery_enabled: CFBooleanRef = unsafe {
        core_foundation::dictionary::CFDictionaryGetValue(
            proxies_dict,
            auto_proxy_discovery_key.as_concrete_TypeRef(),
        )
    };
    if !auto_proxy_discovery_enabled.is_null() {
        let enabled = CFBoolean::wrap_under_get_rule(auto_proxy_discovery_enabled).to_bool();
        if enabled {
            log_warning!("Auto Proxy Discovery is enabled, but not directly supported for proxy URL resolution");
        }
    }

    // Check for PAC (Proxy Auto-Config) proxy
    let pac_proxy: CFStringRef = unsafe {
        core_foundation::dictionary::CFDictionaryGetValue(
            proxies_dict,
            pac_key.as_concrete_TypeRef(),
        )
    };
    if !pac_proxy.is_null() {
        let pac_url = CFString::wrap_under_get_rule(pac_proxy);
        let pac_url_str = pac_url.to_string();
        if !pac_url_str.is_empty() {
            log_info!("Found PAC proxy: {}", pac_url_str);
            return Some(pac_url_str);
        }
    }

    // Helper function to extract proxy URL and port
    fn get_proxy_url(
        proxies_dict: CFDictionaryRef,
        proxy_key: CFStringRef,
        port_key: CFStringRef,
    ) -> Option<String> {
        let proxy: CFStringRef =
            unsafe { core_foundation::dictionary::CFDictionaryGetValue(proxies_dict, proxy_key) };
        if !proxy.is_null() {
            let proxy_str = CFString::wrap_under_get_rule(proxy).to_string();
            if !proxy_str.is_empty() {
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
    if let Some(https_proxy) = get_proxy_url(
        proxies_dict,
        https_proxy_key.as_concrete_TypeRef(),
        https_port_key.as_concrete_TypeRef(),
    ) {
        log_info!("Found HTTPS proxy: {}", https_proxy);
        return Some(format!("https://{}", https_proxy));
    }

    // Check for HTTP proxy
    if let Some(http_proxy) = get_proxy_url(
        proxies_dict,
        http_proxy_key.as_concrete_TypeRef(),
        http_port_key.as_concrete_TypeRef(),
    ) {
        log_info!("Found HTTP proxy: {}", http_proxy);
        return Some(format!("http://{}", http_proxy));
    }

    // Check for SOCKS proxy
    if let Some(socks_proxy) = get_proxy_url(
        proxies_dict,
        socks_proxy_key.as_concrete_TypeRef(),
        socks_port_key.as_concrete_TypeRef(),
    ) {
        log_info!("Found SOCKS proxy: {}", socks_proxy);
        return Some(format!("socks5://{}", socks_proxy));
    }

    // No proxy found
    log_info!("No applicable proxy found for the given URL");
    None
}
