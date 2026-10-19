# codex-switch 集成核验记录

核验时间：2026-10-16。参考项目：https://github.com/xjoker/codex-switch 。

GitHub ZIP 的归档注释确认提交号为 `fe14af0d6b9317a464493816f8bd074eccfae785`。正式随包 CLI 使用该固定提交；固定提交归档 SHA-256 为 `1EDD196C0C10B41552CFC4545A81591492465D88BA3B535D0B8D5F304EFF8CA4`，由 scripts/prepare-cli.ps1 强制校验后编译。

Git clone 在本机两种 TLS 后端均失败；改用 GitHub 官方 codeload 归档成功获取 master 源码。下载归档 SHA-256：`E024469FFAB26FA2D403E3A0825319D7E8BD381F0935563A2786C875ABD8A506`。归档内 Cargo.toml 版本为 `20261001.5.0`，Rust 最低版本 1.88，许可证为 MIT（Copyright 2025 xjoker）。参考树与本地账号文件均排除在提交范围之外。

## 已阅读的协议与行为

- README.md / README_CN.md、Cargo.toml：Windows 支持、文件凭据存储要求、安装方法、上游 Apache-2.0 资产声明。
- src/cli.rs：`--json list`；`list --force` 绕过缓存，因此日常刷新不加 force。TUI 作为独立终端程序运行。
- src/output.rs：顶层 `profiles` 数组；账号 `alias` / `is_current`；成功 usage 含 `fetched_at`、可空 `primary` / `secondary`，窗口含 `label`、`used_percent`、`resets_at`（Unix 秒）。失败 usage 使用 `error`。
- src/commands/profile.rs：账号列表并发刷新；读取缓存后按需联网；list 会跟踪当前账号，可能刷新并保存过期 Token。
- src/config.rs / src/cache.rs：默认缓存 TTL 300 秒。应用每 60 秒调度读取并保留该缓存，不强制重复网络请求。
- src/profile.rs / src/auth.rs：账号切换带事务与备份；删除当前账号被拒绝，其他账号归档到 deleted；写凭据使用受限 ACL 的临时文件原子替换。
- src/commands/import.rs：导入涉及远端验证与 refresh token 轮换，可能恢复已轮换凭据。GUI 不应自行实现 Token 刷新，或在半途中假装回滚已完成的远端轮换。
- src/tui：Accounts、Provider、Settings 页面与独立终端事件循环；不在 WebView 内模拟 TUI。

## 实际运行

已从参考源码执行 `cargo build --locked`，随后运行真实 CLI：

- 独立 CODEX_HOME / CODEX_SWITCH_HOME：`--json list` 返回 `{"profiles":[]}`。
- 当前账号：查询退出码为 0，有 1 个当前账号，5h 与 7d 窗口均存在，usage 无错误。诊断仅保留布尔值与计数。
- 显式 live Rust 集成测试验证 `QuotaProvider` 能取得两个窗口与可靠更新时间。
- v0.0.3 桌面自检验证 WebView IPC、主窗口、托盘、两条实际 DOM 进度条均有效。

## 协议限制

该版本的上游 `window_to_json` 对其内部缺失的 used_percent 使用 0.0 默认值。因此 CLI 输出本身不能区分上游缺失值与真实 0；应用会保留 CLI 提供的百分比，但不额外推断满额。应用自己的缺失/null/非有限/越界输入仍被判为暂无数据。若上游修改此行为，需同步真实回归样本后再提升兼容版本。

解析测试中的非空 JSON 为依据上述真实源码字段构造的协议 fixture，不冒称真实账号抓包；真实账号测试独立显式执行，不将账号数据保存进仓库。
