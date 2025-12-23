# proxyparser

[![Crates.io](https://img.shields.io/crates/v/proxyparser.svg)](https://crates.io/crates/proxyparser)
[![Downloads](https://img.shields.io/crates/d/proxyparser.svg)](https://crates.io/crates/proxyparser)
[![CI](https://github.com/l16659/rustproxyparser/actions/workflows/ci.yml/badge.svg)](https://github.com/l16659/rustproxyparser/actions)

A cross-platform system proxy detector written in Rust, with **strong support for complex enterprise PAC scripts**.

Currently fully supports **macOS** (manual + automatic proxy configuration).  
Windows and Linux support is actively in development.

## Features

- Accurate system proxy detection across platforms
  - macOS: Complete implementation using SystemConfiguration / CoreFoundation
  - Windows: WinHTTP API (in progress)
  - Linux: GNOME/KDE config reading (in progress)

- Full-featured PAC script support
  - Automatic download of PAC files (http/https)
  - Pure-Rust JavaScript execution via `boa_engine`
  - Implements all standard PAC functions (`dnsResolve`, `myIpAddress`, `isInNet`, `shExpMatch`, etc.)
  - Handles complex enterprise scenarios (Zscaler, corporate networks, etc.)

- Correct proxy priority order
  - Environment variables (`HTTP_PROXY`, etc.) → System proxy → DIRECT

- Lightweight binary (~4.7 MB release build, heavily optimized)

## Installation

```bash
cargo install proxyparser


# Query the proxy for a specific URL
proxyparser https://httpbin.org/ip

# Use the default target (google.com) if no URL is provided
proxyparser

git clone https://github.com/l16659/rustproxyparser.git
cd rustproxyparser
cargo build --release
./target/release/proxyparser https://example.com

# Current Status
This project is under active development and maintenance.

# License
This project is licensed under the MIT License.
