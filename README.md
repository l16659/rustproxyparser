# rustproxyparser

## 总体架构

- **Rust cdylib**  
  核心库实现，使用 `#[no_mangle] extern "C"` 导出与现有 **完全一致** 的 C 接口。

- **C 适配层**  
  极薄的 `c_abi` 模块，负责将 C 指针/缓冲区与 Rust 安全类型互相转换。

- **平台抽象**  
  - `macos` 模块封装 SystemConfiguration / CoreFoundation  
  - 其他平台预留接口或维持现有行为

## 运行时与功能模块

| 模块         | 功能描述 |
|--------------|----------|
| **pac_source**   | PAC 源获取（URL、系统配置、内嵌、WPAD） |
| **pac_js**       | JS 引擎封装 + 扩展函数注册（dnsResolve、myIpAddress 等） |
| **pac_eval**     | `FindProxyForURL` 调用与错误处理 |
| **normalize**    | 输出规范化、协议补全（http/socks/direct）、大小写处理、分号/空格解析 |
| **filtering**    | HTTP/SOCKS 类型过滤、最多条数限制、保持顺序 |
| **net**          | DNS 解析、本机 IP 查询，支持返回单条或多条 |
| **logging**      | 可配置日志等级，C 侧沿用现有 `pp_log` 风格或桥接到原有输出 |
| **errors**       | 统一错误类型，映射到当前 C 返回码 |

## 测试层

- Rust 单元测试覆盖各模块  
- Rust 集成测试加载现有 `.pac` 文件，校验与当前行为一致  
- 继续保留 shell 测试用于 CI，对比输出

## JS 引擎选择与策略

- **首选**：**QuickJS**（轻量、支持现代特性、API 简洁）
- **为确保行为不变，建议分两阶段实施**：
  1. **初期**使用 Duktape（已有 Rust 绑定或 C shim），快速实现等价行为，跑通全部现有测试；
  2. **后期**切换到 QuickJS，使用相同的 `.pac` 用例做回归测试，确保兼容性  
     → 如发现难以兼容的问题，可保留 Duktape 作为 fallback 方案。

# Rust Proxy Parser 各模块功能细化（跨 Windows / macOS / Linux）

### pac_source
- 从 URL 获取 PAC 脚本（支持 http、https、file 协议）
- 系统代理配置读取
  - Windows：读取 IE/Edge 系统代理设置 + WPAD（通过 DHCP/DNS）
  - macOS：使用 System Configuration（SCDynamicStoreCopyProxies）
  - Linux：优先读取环境变量（http_proxy、HTTPS_PROXY、no_proxy），其次读取 GSettings（GNOME）、KDE 配置或 systemd
- WPAD 自动发现（DHCP option 252 → DNS 查询 wpad 域名）
- 内嵌默认 PAC 脚本作为最终 fallback
- 优先级顺序：显式 URL > 系统代理配置 > WPAD > 内嵌脚本

### pac_js
- JS 引擎初始化（首选 QuickJS，轻量且支持 ES2023；保留 Duktape 作为兼容后备）
- 注册全部 PAC 标准扩展函数
  - `dnsResolve(host)`
  - `myIpAddress()` / `myIpAddressEx()`
  - `dnsDomainIs`, `isInNet`, `isResolvable`, `isPlainHostName`
  - `shExpMatch`
  - `weekdayRange`, `dateRange`, `timeRange`
- 安全沙箱机制
  - 全局对象只读，禁用 `eval` / `new Function`
  - 设置最大执行时间（默认 5s）和最大调用栈深度
  - 超时或异常统一返回安全结果

### pac_eval
- 加载并预编译 PAC 脚本（支持缓存，相同脚本只编译一次）
- 安全调用 `FindProxyForURL(url, host)`
- 捕获 JS 异常、超时、栈溢出、内存超限等错误
- 错误统一返回 `"DIRECT"` 或自定义错误字符串（如 `"ERR_TIMEOUT"`）
- 支持配置执行超时时间（默认 5s）

### normalize
- 按 `;` 分割多条代理规则
- 去除每条规则前后空格，统一关键字大小写
- 裸 IP:port 自动补全为 `PROXY ip:port`
- 兼容旧版 `SOCKS` → 自动转为 `SOCKS5`
- 特殊值处理：`DIRECT`、`PROXY ERR`、`ERR_xxx`
- 丢弃明显无效或格式错误的条目

### filtering
- 按用户配置过滤协议类型（可只保留 HTTP/HTTPS/SOCKS5/SOCKS4）
- 最大返回条数限制（默认最多 5 条，可配置）
- 严格保持原始顺序（不排序）
- 相同 host+port+type 的条目自动去重
- 若全部被过滤或无有效代理，自动返回单条 `DIRECT`

### net
- `dnsResolve(host)`：跨平台同步/异步 DNS 解析（支持 IPv4 + IPv6 双栈）
- `myIpAddress()`：获取本机首选外出 IP（统一使用 UDP connect 8.8.8.8 方案）
- `myIpAddressEx()`：返回所有非 loopback IP（格式：IPv4;IPv6;...）
- 支持自定义 DNS 服务器列表（优先级高于系统）
- 所有网络操作超时 3s，失败返回空字符串

### logging
- 基于 `tracing` + `tracing-subscriber` 实现
- 支持五级日志：ERROR / WARN / INFO / DEBUG / TRACE
- C 侧提供回调函数 `pp_log(level, msg)`，完全兼容原有日志风格
- 可桥接到平台原生日志系统
  - Windows：OutputDebugString
  - macOS：os_log / NSLog
  - Android：__android_log_write
- Release 模式可通过 feature 完全去除日志

### errors
- 统一错误类型 `enum ProxyError`
  - Io, Network, JsEval, Timeout, InvalidPac, UnsupportedFeature 等
- 实现完整的 `From` 转换链
- C 接口返回原有错误码（0=成功，-1/-2/…=失败）
- 提供 `result_to_c_error()` 映射表，确保与旧版错误码 100% 一致
