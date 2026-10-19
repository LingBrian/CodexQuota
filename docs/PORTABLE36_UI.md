# v0.0.36 额度组件可读性改进

根据截图反馈，将默认组件扩大至 420×48，放大周期标签、百分比、分段进度及重置日期时间区域。两行仍沿任务栏左侧并列垂直排列，重置时间保持可见。

新增亮色透明、暗色透明、亮色不透明、暗色不透明预设。预设会同时切换背景、透明度和高对比文字颜色；原显示器、任务栏偏移、点击穿透偏好不会被更改。旧配置的紧凑布局仅迁移一次，扩大组件尺寸并保留位置偏好。

验证结果：前端 22 项通过；Rust 41 项通过，1 项本机登录凭据测试忽略；`npm run check`、`cargo fmt --check`、Clippy 通过。最终 release 可执行文件便携冒烟报告 `reports/portable36-final/runtime-smoke.json`：`passed=true`、`overlayGeometry.size=420×48`、`overlayInTaskbar=true`、`overlayLayoutValid=true`、`backgroundMatches=true`、`resetTimesVisible=true`。
