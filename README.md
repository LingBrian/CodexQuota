# CodexQuota

Windows 上的 Codex 额度监控桌面应用，采用 **Tauri 2 + Vue 3 + TypeScript + Rust**。默认界面为简体中文，不上传认证凭据、不收集遥测数据。

当前版本为 **v0.0.41**。版本按 `v0.0.N` 管理，M1–M30 的交付范围与门禁见 [开发路线图](docs/ROADMAP.md)；测试开发规则见 [测试策略](docs/TEST_STRATEGY.md)。阶段运行报告保存在本地 reports 目录，发布前还需完成跨机器验收矩阵。

真实额度查询已接通，已在 Windows 11 上启动桌面应用验证 5h/7d 两项额度、托盘与任务栏组件。本项目集成的上游 CLI 参考仓库为 [xjoker/codex-switch](https://github.com/xjoker/codex-switch)；固定提交、JSON 协议和已知限制见[参考项目研究](docs/UPSTREAM_RESEARCH.md)。

当前发布版本为 **v0.0.41**。界面与任务栏预设见 [UI 改版说明](docs/UI_REDESIGN.md)。逐阶段报告、验收状态、安全检查及安装说明分别见 [阶段报告](docs/STAGE_RUNS.md)、[验收状态](docs/VERIFICATION.md)、[安全检查](docs/SECURITY_REVIEW.md)和[安装说明](docs/INSTALL.md)。未完成的跨环境、截图及长稳项目不宣称通过。

## 功能

- 仪表盘：真实数据优先；字段缺失时显示“暂无数据”。
- 刷新状态、数据来源与错误原因展示。
- 任务栏**兼容覆盖层**：无边框、透明、跳过任务栏；不是原生任务栏插件。
- 可保存刷新间隔、覆盖层字体颜色、字号和透明度。
- 托盘图标绘制两条独立额度进度条，未知值为灰色短线；托盘支持刷新、打开账号管理、编辑样式和退出。
- 账号导入预览、冲突检测、确认切换、重命名、可恢复删除与归档恢复。
- 查询超时/取消、刷新去重、缓存持久化和错误时保留上次成功数据。
- 5h/7d 独立提醒阈值、跨午夜免打扰、提醒去重、恢复和重置提醒。
- Rust 后端限定外部可执行文件名，拒绝任意 shell 参数执行。
- 中英文、系统／深浅主题，强调色、显示器和组件完整样式配置。
- 设置导入导出、未来版本只读保护、脱敏诊断与轮转日志。
- 当前用户开机启动（默认关闭）；更新清单仅格式预检，不自动安装。
- 安装包附带固定版本、安全补丁后的 CLI 与第三方许可证。

## 快速开始（Windows）

先安装 Node.js 22.22.2+、Rust stable、Visual Studio Build Tools（Desktop development with C++）与 WebView2 Runtime：

```powershell
git clone https://github.com/LingBrian/CodexQuota.git CodexQuota
cd CodexQuota
npm install
powershell -File scripts/prepare-cli.ps1
npm run tauri dev
```

构建安装包：

```powershell
npm run tauri build
```

输出位于 `src-tauri/target/release/bundle/nsis` 与 `src-tauri/target/release/bundle/msi`（MSI 取决于本机 WiX 环境）。未配置代码签名时，Windows SmartScreen 可能警告；本项目不声称已签名。

## 额度数据与安全

额度适配器使用核验过的 `codex-switch --json list`，默认每 60 秒调度读取，保留上游 300 秒缓存。认证文件的导入、Token 刷新与账号切换留在 Rust/CLI 后端；Token 不会进入 Vue 或日志，也不会被上传到 CodexQuota 服务。导入确认后，上游 CLI 会按其认证机制联系 OpenAI 服务验证凭据。

任务栏组件采用贴靠任务栏工作区的兼容窗口模式。使用公开 AppBar/显示器 API 读取位置、按显示器 DPI 缩放，并定时重新定位；不占用任务栏按钮。原生托盘依赖包含 TaskbarCreated 恢复逻辑。Windows 10、物理多显示器、Explorer 重启及自动隐藏任务栏场景需按发布矩阵单独验收；当前机器实测为 Windows 11、单显示器。兼容组件具有透明背景、鼠标穿透选项及不激活显示的 Win32 样式。

## 开发检查

```powershell
npm run check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## 常见问题

**为什么是“暂无数据”？** 未配置/验证上游 JSON 协议、CLI 不存在、超时或登录失效时，应用不会以 100% 替代真实值。

**如何指定 CLI？** 优先使用随包 CLI。也可以在设置中填写已安装的 `codex-switch.exe` 完整路径。参数由后端固定，不支持任意 shell 命令。

**为什么不是原生任务栏组件？** Windows 没有稳定公开的第三方任务栏嵌入 API；覆盖层是兼容实现，文档中已明确标注。

## 截图

当前仓库未附带桌面截图；截图、跨环境和长稳验收状态见[验收记录](docs/VERIFICATION.md)。请勿提交包含账号、Token 或本地路径的截图。

## 许可证

本项目使用 [MIT](LICENSE)。第三方组件与参考项目使用情况见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
