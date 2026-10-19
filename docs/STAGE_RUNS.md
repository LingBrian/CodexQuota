# 逐阶段桌面启动证据

以下均为真实桌面进程自检报告，不是浏览器模拟额度。原始报告在执行测试的本机保留；为避免公开运行时数据或账号快照，`reports/` 被 Git 忽略且不随公开源码发布。下表记录本地报告路径，克隆仓库中不会包含这些 JSON。测试通过不代表物理硬件／跨系统矩阵已通过。

| 阶段 | 版本 | 最近一次通过报告 |
|---|---|---|
| M1 | v0.0.1 | 未保留独立桌面报告；基础能力在后续全量回归中覆盖，不补造历史记录 |
| M2 | v0.0.2 | 未保留独立桌面报告；基础能力在后续全量回归中覆盖，不补造历史记录 |
| M3 | v0.0.3 | reports/M3-first-run/runtime-smoke.json |
| M4 | v0.0.4 | reports/M4-20261016T024949/runtime-smoke.json |
| M5 | v0.0.5 | reports/M5-20261016T025156/runtime-smoke.json |
| M6 | v0.0.6 | reports/M6-20261016T025257/runtime-smoke.json |
| M7 | v0.0.7 | reports/M7-20261016T025403/runtime-smoke.json |
| M8 | v0.0.8 | reports/M8-20261016T025514/runtime-smoke.json |
| M9 | v0.0.9 | reports/M9-20261016T031006/runtime-smoke.json |
| M10 | v0.0.10 | reports/M10-20261016T031207/runtime-smoke.json |
| M11 | v0.0.11 | reports/M11-20261016T031320/runtime-smoke.json |
| M12 | v0.0.12 | reports/M12-20261016T031436/runtime-smoke.json |
| M13 | v0.0.13 | reports/M13-20261016T031548/runtime-smoke.json |
| M14 | v0.0.14 | reports/M14-20261016T031701/runtime-smoke.json |
| M15 | v0.0.15 | reports/M15-20261016T031815/runtime-smoke.json |
| M16 | v0.0.16 | reports/M16-20261016T031927/runtime-smoke.json |
| M17 | v0.0.17 | reports/M17-20261016T033810/runtime-smoke.json |
| M18 | v0.0.18 | reports/M18-20261016T034052/runtime-smoke.json |
| M19 | v0.0.19 | reports/M19-20261016T034241/runtime-smoke.json |
| M20 | v0.0.20 | reports/M20-20261016T034948/runtime-smoke.json |
| M21 | v0.0.21 | reports/M21-20261016T035136/runtime-smoke.json |
| M22 | v0.0.22 | reports/M22-background-notification/runtime-smoke.json |
| M23 | v0.0.23 | reports/M23-20261016T040512/runtime-smoke.json |
| M24 | v0.0.24 | reports/M24-20261016T040722/runtime-smoke.json |
| M25 | v0.0.25 | reports/M25-20261016T041034/runtime-smoke.json |
| M26 | v0.0.26 | reports/M26-20261016T041245/runtime-smoke.json |
| M27 | v0.0.27 | reports/M27-20261016T041546/runtime-smoke.json |
| M28 | v0.0.28 | reports/M28-20261016T041811/runtime-smoke.json |
| M29 | v0.0.29 | reports/M29-20261016T042228/runtime-smoke.json |
| M30 | v0.0.30 | reports/M30-release-runtime/runtime-smoke.json |

M28 的桌面自检不替代 24 小时长稳；M30 仍需安装／卸载、跨环境和稳定性门禁。Windows 10、物理第二显示器按用户要求保留待验收。
