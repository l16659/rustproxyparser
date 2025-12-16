# rustproxyparser

A cross-platform system proxy detector written in Rust, with strong support for complex enterprise PAC scripts.

The current version fully supports macOS (including manual proxies and automatic proxy configuration). Windows and Linux support is in progress.

## Features

- **Accurate system proxy detection**
  - macOS: Complete implementation using SystemConfiguration / CoreFoundation
  - Windows: Based on WinHTTP API (in development)
  - Linux: Supports GNOME/KDE configuration reading (in development)

- **Full PAC script support**
  - Automatic download of PAC files (http/https)
  - JavaScript execution using pure Rust `boa_engine`
  - Implements all common PAC functions:
    - `dnsResolve`, `myIpAddress`, `isInNet`, `isResolvable`
    - `shExpMatch`, `dnsDomainIs`, `localHostOrDomainIs`
    - `dnsDomainLevels`, `weekdayRange`, etc.
  - Handles complex enterprise PAC scripts (e.g., Zscaler, corporate networks)

- **Correct priority order**
  - Environment variables (HTTP_PROXY, etc.) > System proxy > DIRECT

- **Lightweight and efficient**
  - Release build size ~4.7MB (heavily optimized)

## Usage

```bash
# Build
cargo build --release

# Query proxy for a specific URL
./target/release/proxyparser https://httpbin.org/ip

# Default: queries google.com
./target/release/proxyparser

```

## 🚧 Under Maintenance & Development
This project is currently under active maintenance and iterative development. 

Stay tuned for the next release!
