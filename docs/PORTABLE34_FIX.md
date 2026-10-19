# v0.0.34 任务栏内定位与显隐

旧实现 near_edge 使用工作区矩形，实际结果在任务栏上方。新版默认使用 SHAppBarMessage 返回的任务栏矩形；非主屏根据工作区与屏幕差额定位，有效范围内缩放宽高并沿栏定位。桌面浮窗提供独立横纵偏移。

显示和隐藏统一使用原生 ShowWindow，消除原生显示与框架显隐状态缓存混用。位置修改与窗口变更串行处理；前端保存排队，较早响应不覆盖较新的输入。设置页预览也响应隐藏状态。

20 项前端和 40 项 Rust 测试通过；1 项真实登录专项默认忽略。首次任务栏测试已证明 overlayInTaskbar、visibilityVerified、positionSettingsApplied 均为 true，但穿透自检被定时恢复覆盖，整组判失败。已将穿透自检改为真实保存设置流程，不再仅临时修改原生窗口样式。

原生点击／截图验收仍未完成。这里的任务栏内模式是独立覆盖窗口，不改 Explorer 窗口树、不占用系统原生工具栏槽位。

最终报告：reports/portable34-taskbar-final/runtime-smoke.json 与 reports/portable34-desktop/runtime-smoke.json 均 passed=true。任务栏范围 Y=1032..1080，窗口 300×46 位于 (76,1033)，overlayInTaskbar=true；浮窗在 (350,180)，overlayInWorkArea=true。两者 visibilityVerified、positionSettingsApplied、interactionVerified、querySucceeded 均为 true。
