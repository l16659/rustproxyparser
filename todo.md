问题,严重性,说明,建议修复方案
1. PAC 发现与执行路径不完整,高,目前 macOS/Linux 发现 PAC URL 后只 log，没下载也没执行；Windows 有框架但未实现,必须补全 PAC 下载 + JS 执行链
2. find_proxy_for_url 被定义了两次,中,interface.rs 和 main.rs 里都定义了相同函数 → 重复且可能冲突,删除 main.rs 中的重复定义，只保留一个公共接口
3. system_proxy::mod.rs 暴露了 get_system_proxy，但 interface 未使用它,中,导致 interface.rs 里又重复写了平台判断,统一走 system_proxy::get_system_proxy
4. Linux 实现依赖 sys-proxy crate 作为 fallback，但它不一定准确,中,sys-proxy 只是猜测，不读真实 GNOME/KDE 设置,你已经手动读 gsettings/kreadconfig5，更准确，应优先使用
5. 返回的代理格式不统一,高,"macOS 返回 ""https://host:port""，但其他地方可能是裸 ""host:port"" 或 ""http://...""",必须统一规范化为带 scheme 的完整 URL 或标准 PAC 格式
6. 未实现代理结果的后处理（normalize + filtering）,高,README 中提到的重要功能：去重、协议过滤、最大条数、SOCKS→SOCKS5、DIRECT 处理等,这是你最终目标的核心
