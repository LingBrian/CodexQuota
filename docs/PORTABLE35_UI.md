# v0.0.35 截图反馈修复

根据图二将组件改为左贴齐双行，每行依次显示周期、分段剩余进度、百分比、重置日期时间。两行垂直居中于任务栏组件窗口。窗口从任务栏左沿开始，默认不再留旧胶囊的水平内边距。

时间始终显示，旧的 showCountdown=false 不再将其隐藏；日期和时间直接取额度 API resetAt，本地化为 MM/DD HH:mm。悬停提示仍显示剩余倒计时。

背景颜色使用保存在设置中的 background 和 backgroundOpacity。样式预设不再把用户自选背景色改回默认色。位置与交互卡片可继续设置颜色和透明度。

验证：21 项前端、40 项 Rust 测试通过，1 项登录凭据测试默认忽略；类型检查、Vite/Rust production build、fmt 和 Clippy 通过。`reports/portable35/runtime-smoke.json` 中 `passed=true`、`overlayLayoutValid=true`、`backgroundMatches=true`、`resetTimesVisible=true`、`overlayInTaskbar=true`、`positionSettingsApplied=true`。当前窗口坐标 (0,1033)，任务栏区域从 y=1032 开始。
