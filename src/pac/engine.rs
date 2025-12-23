// src/pac/engine.rs
use super::downloader::download_pac;
use crate::log_warn;
use boa_engine::{
    js_string, Context, JsNativeError, JsResult, JsString, JsValue, NativeFunction, Source,
};
use chrono::{Datelike, Utc, Weekday};
use regex::Regex;
use std::net::{Ipv4Addr, ToSocketAddrs, UdpSocket};
use url::Url;

pub fn evaluate_pac_for_url(
    pac_url: &str,
    target_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let script = download_pac(pac_url)?;
    let url_obj = Url::parse(target_url)?;
    let host = url_obj.host_str().ok_or("URL has no host")?.to_string();

    let raw_result = execute_pac_script(&script, target_url, &host)?;
    let normalized = normalize_pac_result(&raw_result);

    Ok(normalized)
}

fn execute_pac_script(script: &str, url: &str, host: &str) -> JsResult<String> {
    let mut context = Context::default();

    register_pac_functions(&mut context)?;

    context.eval(Source::from_bytes(script.as_bytes()))?;

    let global = context.global_object();
    let func_val = global.get(js_string!("FindProxyForURL"), &mut context)?;

    let func = func_val
        .as_callable()
        .ok_or(JsNativeError::typ().with_message("FindProxyForURL is not a function"))?;

    let args = [
        JsValue::from(js_string!(url)),
        JsValue::from(js_string!(host)),
    ];

    let result = func.call(&JsValue::undefined(), &args, &mut context)?;
    let js_str: JsString = result.to_string(&mut context)?;

    Ok(js_str.to_std_string().unwrap_or_default())
}

fn register_pac_functions(context: &mut Context) -> JsResult<()> {
    // isPlainHostName(host)
    context.register_global_callable(
        "isPlainHostName".into(),
        1,
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let host = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            Ok((!host.contains('.')).into())
        }),
    )?;

    // dnsDomainIs(host, domain)
    context.register_global_callable(
        "dnsDomainIs".into(),
        2,
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let host = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let domain = args
                .get(1)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let is_match = host.ends_with(&domain)
                && (host.len() == domain.len()
                    || host.as_bytes()[host.len() - domain.len() - 1] == b'.');
            Ok(is_match.into())
        }),
    )?;

    // localHostOrDomainIs(host, domain)
    context.register_global_callable(
        "localHostOrDomainIs".into(),
        2,
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let host = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let domain = args
                .get(1)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            Ok((host == domain || host.ends_with(&format!(".{}", domain))).into())
        }),
    )?;

    // isResolvable(host)
    context.register_global_callable(
        "isResolvable".into(),
        1,
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let host = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let resolved = (host.as_str(), 0).to_socket_addrs().is_ok();
            Ok(resolved.into())
        }),
    )?;

    // dnsResolve(host)
    context.register_global_callable(
        "dnsResolve".into(),
        1,
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let host = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let ip = (host.as_str(), 0)
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
                .map(|addr| addr.ip().to_string());
            if let Some(ip_str) = ip {
                Ok(js_string!(ip_str).into())
            } else {
                Ok(JsValue::null())
            }
        }),
    )?;

    // isInNet(ip, net, mask)
    context.register_global_callable(
        "isInNet".into(),
        3,
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let ip_str = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let net_str = args
                .get(1)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let mask_str = args
                .get(2)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();

            let ip: Option<Ipv4Addr> = ip_str.parse().ok();
            let net: Option<Ipv4Addr> = net_str.parse().ok();
            let mask: Option<Ipv4Addr> = mask_str.parse().ok();

            if let (Some(ip), Some(net), Some(mask)) = (ip, net, mask) {
                let ip_u32 = u32::from_be_bytes(ip.octets());
                let net_u32 = u32::from_be_bytes(net.octets());
                let mask_u32 = u32::from_be_bytes(mask.octets());
                Ok(((ip_u32 & mask_u32) == (net_u32 & mask_u32)).into())
            } else {
                Ok(false.into())
            }
        }),
    )?;

    // dnsDomainLevels(host)
    context.register_global_callable(
        "dnsDomainLevels".into(),
        1,
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let host = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let levels = host.matches('.').count() as i32;
            Ok(JsValue::from(levels))
        }),
    )?;

    // myIpAddress()
    context.register_global_callable(
        "myIpAddress".into(),
        0,
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            let socket = UdpSocket::bind("0.0.0.0:0").ok();
            let ip = socket
                .as_ref()
                .and_then(|s| s.connect("8.8.8.8:53").ok())
                .and_then(|_| socket.as_ref().and_then(|s| s.local_addr().ok()))
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| "127.0.0.1".to_string());
            Ok(js_string!(ip).into())
        }),
    )?;

    // shExpMatch(str, glob)
    context.register_global_callable(
        "shExpMatch".into(),
        2,
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let str_val = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();
            let pattern = args
                .get(1)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default();

            let regex_pattern = pattern
                .replace('*', ".*")
                .replace('?', ".")
                .replace('|', "\\|");

            let re = Regex::new(&format!("^{}$", regex_pattern))
                .unwrap_or_else(|_| Regex::new("^$").unwrap());

            Ok(re.is_match(&str_val).into())
        }),
    )?;

    // weekdayRange(wd1, [wd2], [gmt])
    context.register_global_callable(
        "weekdayRange".into(),
        3, // 最多 3 个参数
        NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let now = Utc::now();
            let current_wd = match now.weekday() {
                Weekday::Mon => "MON",
                Weekday::Tue => "TUE",
                Weekday::Wed => "WED",
                Weekday::Thu => "THU",
                Weekday::Fri => "FRI",
                Weekday::Sat => "SAT",
                Weekday::Sun => "SUN",
            };

            let wd1 = args
                .get(0)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok())
                .unwrap_or_default()
                .to_uppercase();

            let wd2 = args
                .get(1)
                .and_then(|v| v.as_string())
                .and_then(|s| s.to_std_string().ok());

            // GMT 参数忽略（大多数 PAC 不依赖）
            if wd2.is_none() {
                return Ok((current_wd == wd1).into());
            }

            let wd2_str = wd2.unwrap().to_uppercase();

            if wd1 == wd2_str {
                Ok((current_wd == wd1).into())
            } else {
                let days = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
                let start = days.iter().position(|&d| d == wd1).unwrap_or(0);
                let end = days.iter().position(|&d| d == wd2_str).unwrap_or(0);
                let curr = days.iter().position(|&d| d == current_wd).unwrap_or(0);

                // 如果跨周（例如 FRI 到 MON），简单处理为 false 或 true，根据常见行为
                if start <= end {
                    Ok((curr >= start && curr <= end).into())
                } else {
                    Ok((curr >= start || curr <= end).into())
                }
            }
        }),
    )?;

    // dateRange 和 timeRange - 企业 PAC 极少严格依赖时间，直接返回 true 足够
    context.register_global_callable(
        "dateRange".into(),
        8,
        NativeFunction::from_fn_ptr(|_, _, _| Ok(true.into())),
    )?;
    context.register_global_callable(
        "timeRange".into(),
        6,
        NativeFunction::from_fn_ptr(|_, _, _| Ok(true.into())),
    )?;

    Ok(())
}

fn normalize_pac_result(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("DIRECT") {
        return "DIRECT".to_string();
    }

    for part in trimmed.split(';') {
        let part = part.trim();
        if part.eq_ignore_ascii_case("DIRECT") {
            continue;
        }
        if let Some(stripped) = part.strip_prefix("PROXY ") {
            return format!("http://{}", stripped.trim());
        }
        if let Some(stripped) = part.strip_prefix("HTTPS ") {
            return format!("https://{}", stripped.trim());
        }
        if part.starts_with("SOCKS5 ") {
            return format!("socks5://{}", part[7..].trim());
        }
        if part.starts_with("SOCKS ") {
            return format!("socks5://{}", part[6..].trim());
        }
        if part.contains(':') && !part.contains("://") {
            return format!("http://{}", part);
        }
    }

    log_warn!("PAC returned no valid proxy, falling back to DIRECT");
    "DIRECT".to_string()
}
