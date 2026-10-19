<div align="center">

# CodexQuota

**把 Codex 额度放在眼前。**

Windows 桌面额度监控工具，集中查看 5 小时与 7 天额度、管理账号，并把轻量进度组件放在任务栏旁。

[![CI](https://github.com/LingBrian/CodexQuota/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/LingBrian/CodexQuota/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v0.0.41-orange)](https://github.com/LingBrian/CodexQuota/releases/tag/v0.0.41)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Windows](https://img.shields.io/badge/platform-Windows%2010%2B-0078D4)](docs/INSTALL.md)

[下载 v0.0.41（预发布）](https://github.com/LingBrian/CodexQuota/releases/tag/v0.0.41) · [安装指南](docs/INSTALL.md) · [开发路线图](docs/ROADMAP.md) · [问题反馈](https://github.com/LingBrian/CodexQuota/issues)

</div>

> 当前版本为早期预发布版本。Windows 11 单显示器环境已完成基础运行验证；Windows 10、物理多显示器及长时间稳定性仍待验收。请先阅读[验收状态](docs/VERIFICATION.md)。

## 一眼了解

| | |
|---|---|
| **应用框架** | Tauri 2 · Vue 3 · TypeScript · Rust |
| **支持平台** | Windows 桌面端 |
| **当前版本** | `v0.0.41`（预发布） |
| **上游参考** | [xjoker/codex-switch](https://github.com/xjoker/codex-switch)，固定版本与协议说明见[研究记录](docs/UPSTREAM_RESEARCH.md) |
| **许可证** | [MIT](LICENSE) |

## 功能亮点

| 额度与提醒 | 账号管理 |
|---|---|
| 展示 5 小时与 7 天额度、刷新状态、数据来源和错误原因 | 导入预览、冲突检测、确认切换、重命名与可恢复删除 |
| 数据缺失时明确显示“暂无数据”，不伪造额度 | 支持归档恢复与账号配置管理 |
| 独立提醒阈值、免打扰时段、去重及恢复提醒 | Token 操作保留在 Rust/CLI 后端 |

| 桌面体验 | 稳定性与隐私 |
|---|---|
| 托盘图标绘制两条独立额度进度条 | 查询超时/取消、刷新去重、缓存和错误恢复 |
| 可配置任务栏旁透明覆盖层、字号、颜色和透明度 | 后端限制可执行文件与参数，不执行任意 shell 命令 |
| 中英文、系统/深浅主题、多显示器与完整样式设置 | Token 不进入前端或日志；应用不上传凭据、不收集遥测 |

任务栏组件是贴靠任务栏工作区的**兼容覆盖层**，不是 Windows 原生任务栏插件。它不占用任务栏按钮；不同 Windows 版本、显示器布局及自动隐藏模式的兼容性请查看[验收记录](docs/VERIFICATION.md)。

## 下载与运行

前往 [v0.0.41 发布页](https://github.com/LingBrian/CodexQuota/releases/tag/v0.0.41)，按需下载安装包或便携包。建议使用发布页提供的 SHA-256 文件校验下载完整性。安装、便携版和常见问题说明见[安装指南](docs/INSTALL.md)。

未配置代码签名时，Windows SmartScreen 可能显示警告；该版本不声称已签名。首次查询需要可用的 Codex 登录状态。额度查询由随包 CLI 按其认证机制联系 OpenAI 服务；CodexQuota 本身不提供中转服务。

## 从源码运行

需要 Node.js 22.22.2 或更高版本、Rust stable、Visual Studio Build Tools（含 *Desktop development with C++*）和 WebView2 Runtime。

```powershell
git clone https://github.com/LingBrian/CodexQuota.git
cd CodexQuota
npm ci
powershell -ExecutionPolicy Bypass -File scripts/prepare-cli.ps1
npm run tauri dev
```

构建 Windows 安装包：

```powershell
npm run tauri build
```

构建结果位于 `src-tauri/target/release/bundle/`。MSI 是否生成取决于本机 WiX 环境。开发阶段、M1–M30 交付目标与门禁详见[开发路线图](docs/ROADMAP.md)和[测试策略](docs/TEST_STRATEGY.md)。

## 本地检查

```powershell
npm run check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## 数据与安全

额度适配器调用经核验的 `codex-switch --json list`，默认每 60 秒调度读取，并保留上游 300 秒缓存。认证文件导入、Token 刷新与账号切换由 Rust/CLI 后端处理；Token 不传入 Vue 前端或应用日志。导入确认后，上游 CLI 会按自身认证机制验证凭据。

安全审查及已知边界见[安全检查](docs/SECURITY_REVIEW.md)；上游固定提交、JSON 协议和限制见[参考项目研究](docs/UPSTREAM_RESEARCH.md)。发现安全问题时，请避免在公开 Issue 中贴出凭据或个人数据。

## 文档

- [安装指南](docs/INSTALL.md) · [验收状态](docs/VERIFICATION.md) · [安全检查](docs/SECURITY_REVIEW.md)
- [开发路线图](docs/ROADMAP.md) · [测试策略](docs/TEST_STRATEGY.md) · [阶段报告](docs/STAGE_RUNS.md)
- [UI 改版说明](docs/UI_REDESIGN.md) · [上游项目研究](docs/UPSTREAM_RESEARCH.md) · [第三方许可](THIRD_PARTY_NOTICES.md)

## 参与贡献

欢迎通过 [Issues](https://github.com/LingBrian/CodexQuota/issues) 报告问题或提出建议，也欢迎提交 Pull Request。提交前请运行上方本地检查，并遵循仓库中的 [Issue 模板](.github/ISSUE_TEMPLATE) 与 [PR 模板](.github/pull_request_template.md)。涉及界面问题时，请使用脱敏截图，不要包含账号、Token 或本地路径。

## 许可证

CodexQuota 使用 [MIT License](LICENSE)。上游项目及第三方组件许可说明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
