# rustproxyparser

一个跨平台的系统代理探测工具，支持 macOS、Windows 和 Linux，特别擅长处理复杂的企业级 PAC 脚本。

当前版本已完整支持 macOS（包括手动代理和 PAC 自动配置），Windows 和 Linux 正在开发中。

## 特性

- **准确读取系统代理配置**
  - macOS：基于 SystemConfiguration / CoreFoundation 完整实现
  - Windows：基于 WinHTTP API（开发中）
  - Linux：支持 GNOME/KDE 配置读取（开发中）

- **完整 PAC 脚本支持**
  - 自动下载 PAC 文件（支持 http/https）
  - 使用纯 Rust 的 `boa_engine` 执行 JavaScript
  - 已实现所有常见 PAC 函数：
    - `dnsResolve`, `myIpAddress`, `isInNet`, `isResolvable`
    - `shExpMatch`, `dnsDomainIs`, `localHostOrDomainIs`
    - `dnsDomainLevels`, `weekdayRange` 等
  - 支持复杂企业 PAC（如 Zscaler、公司内网脚本）

- **优先级正确**
  - 环境变量（HTTP_PROXY 等） > 系统代理 > DIRECT

- **轻量高效**
  - Release 构建体积约 4.7MB（已优化）

## 使用示例

```bash
# 编译
cargo build --release

# 查询指定 URL 的代理
./target/release/proxyparser https://httpbin.org/ip

# 默认查询 google.com
./target/release/proxyparser
