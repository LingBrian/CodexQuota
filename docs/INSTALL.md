# Windows 安装与升级

当前发布版本为 v0.0.41。跨环境与 24 小时长稳验收状态见 `VERIFICATION.md`；Windows 程序未进行代码签名。新版启动前需退出旧实例，避免单实例机制把你带回旧界面。

开机启动是当前用户显式选择的功能，默认关闭。NSIS 卸载会移除本应用的启动项；使用 MSI 卸载前，请先在应用设置中关闭开机启动。卸载不应清除 Codex 登录文件。

安装包支持 Windows 10/11 x64，需要 Microsoft WebView2 Runtime。发布包应同时包含 NSIS `.exe`、MSI `.msi` 与 `SHA256SUMS.txt`。

1. 对照 SHA256SUMS.txt 验证安装包：`Get-FileHash <安装包路径> -Algorithm SHA256`。
2. 双击其中一种安装包安装，不必重复安装两种格式。
3. 首次启动读取本机 Codex 登录状态，已有文件凭据可由随包 codex-switch 发现。
4. 账号导入时先预览，切换前确认对新 Codex 会话的影响。

应用未配置 Windows 代码签名。SmartScreen 可能显示未识别发布者；不要将本构建描述为已签名。

设置保存在 `%APPDATA%/CodexQuota`，额度缓存与提醒状态保存在 `%LOCALAPPDATA%/CodexQuota`。认证信息由 codex-switch 保存在它自己的本机存储中，CodexQuota 不会将认证内容发送给前端。

关闭窗口后应用继续驻留托盘；退出请使用托盘“退出”。升级前先退出旧版本。卸载应用不应删除用户的 Codex 登录文件或 codex-switch 账号目录。

正式安装/升级/卸载测试应在独立 Windows 10/11 环境执行，避免影响已有工作环境。发布验收报告必须区分安装包已生成、已解包检查和已实际安装三种结果。
