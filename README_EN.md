# CodexQuota

Current release: **v0.0.41**. The taskbar overlay no longer reorders its native window on every foreground-app change; it only restores z-order when the foreground window actually covers it. Cross-machine, screenshot, and long-running acceptance items are documented and are not claimed as passed. Windows binaries are not code-signed.

Codex quota monitor for Windows, built with Tauri 2, Vue 3, TypeScript, and Rust. It is local-first and does not upload credentials or telemetry.

The delivery path from v0.0.1 (M1) through v0.0.30 (M30) is in the [roadmap](docs/ROADMAP.md), with the test-first rules in [the test strategy](docs/TEST_STRATEGY.md).

The application connects to the verified [`codex-switch --json list`](https://github.com/xjoker/codex-switch) contract and has passed live quota and desktop startup tests on Windows 11. It provides persisted settings, a dashboard, two tray progress bars, account import preview and confirmed operations, cached quota fallback, reminders, and a taskbar-compatible overlay. Windows 10 and physical multi-monitor acceptance remain separate release checks. See `docs/UPSTREAM_RESEARCH.md` for the pinned CLI and contract limitations.

## Build

Install Node.js 22.22.2+, Rust stable, Visual Studio Build Tools with C++ desktop tooling, and WebView2 Runtime.

```powershell
npm install
powershell -File scripts/prepare-cli.ps1
npm run tauri dev
npm run tauri build
```

NSIS and MSI bundles are emitted under `src-tauri/target/release/bundle` when prerequisites are available. Binaries are not code-signed; SmartScreen warnings can occur.

No desktop screenshots are included in this repository. Cross-environment and long-running acceptance status is documented in `docs/VERIFICATION.md`.

The overlay is a compatibility window, not a native taskbar plugin. It intentionally avoids DLL injection, Explorer patching and undocumented APIs.

See [README.md](README.md) for Chinese documentation and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for notices.
