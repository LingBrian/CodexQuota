mod accounts;
mod adapter;
mod autostart;
mod cache;
mod config_transfer;
mod diagnostics;
mod notification_setup;
mod process;
mod quota;
mod refresh;
mod reminders;
mod settings;
mod smoke;
mod soak;
mod taskbar;
mod tray;
mod updates;

use quota::{QuotaErrorCode, QuotaProvider, QuotaSnapshot};
use settings::{AppSettings, SettingsStore};
use std::sync::Mutex;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_notification::NotificationExt;

struct State {
    journal: diagnostics::Journal,
    settings: Mutex<SettingsStore>,
    provider: Mutex<QuotaProvider>,
    refresh: refresh::RefreshCoordinator,
    imports: accounts::ImportManager,
    reminders: Mutex<reminders::ReminderState>,
    overlay_ready: std::sync::atomic::AtomicBool,
    overlay_layout_valid: std::sync::atomic::AtomicBool,
    overlay_background_matches: std::sync::atomic::AtomicBool,
    reset_times_visible: std::sync::atomic::AtomicBool,
    background_cycles: std::sync::atomic::AtomicU64,
    exiting: std::sync::atomic::AtomicBool,
    overlay_gate: tokio::sync::Mutex<()>,
}
struct TrayLabels {
    five: MenuItem<tauri::Wry>,
    seven: MenuItem<tauri::Wry>,
    menu: Menu<tauri::Wry>,
}
fn data_directory() -> std::path::PathBuf {
    std::env::var_os("CODEXQUOTA_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("LOCALAPPDATA")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("CodexQuota")
        })
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, State>) -> Result<AppSettings, String> {
    state.settings.lock().map_err(|_| "设置锁不可用")?.get()
}

#[tauri::command]
async fn save_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
    state: tauri::State<'_, State>,
) -> Result<AppSettings, String> {
    let _guard = state.overlay_gate.lock().await;
    settings.validate()?;
    state
        .settings
        .lock()
        .map_err(|_| "设置锁不可用")?
        .save(settings.clone())?;
    app.emit("settings-updated", &settings)
        .map_err(|_| "无法更新窗口设置")?;
    update_overlay(&app, settings.overlay_visible)?;
    let _ = state.journal.record(diagnostics::EventCode::SettingsSaved);
    Ok(settings)
}

#[tauri::command]
async fn refresh_quota(
    app: tauri::AppHandle,
    state: tauri::State<'_, State>,
) -> Result<QuotaSnapshot, String> {
    refresh_application(&app, &state).await
}

async fn refresh_application(
    app: &tauri::AppHandle,
    state: &State,
) -> Result<QuotaSnapshot, String> {
    let config = match state.settings.lock() {
        Ok(s) => s.get().unwrap_or_default(),
        Err(_) => {
            return Ok(QuotaSnapshot::error(
                QuotaErrorCode::Internal,
                "设置锁不可用",
                true,
            ))
        }
    };
    let provider = match state.provider.lock() {
        Ok(p) => p.clone(),
        Err(_) => {
            return Ok(QuotaSnapshot::error(
                QuotaErrorCode::Internal,
                "查询器锁不可用",
                true,
            ))
        }
    };
    let snapshot = state
        .refresh
        .run(
            std::time::Duration::from_secs(10),
            std::time::Duration::from_secs(35),
            || provider.refresh(&config),
        )
        .await;
    let _ = state.journal.record(if snapshot.error.is_none() {
        diagnostics::EventCode::QuotaSucceeded
    } else {
        diagnostics::EventCode::QuotaFailed
    });
    let _ = app.emit("quota-updated", &snapshot);
    {
        let mut reminder_state = state.reminders.lock().map_err(|_| "提醒状态不可用")?;
        let mut candidate = reminder_state.clone();
        let messages = candidate.evaluate(
            &snapshot,
            &config.reminders,
            chrono::Utc::now(),
            reminders::ReminderState::local_hour(),
        );
        let mut delivered = true;
        for message in &messages {
            let builder = app
                .notification()
                .builder()
                .title("CodexQuota")
                .body(message);
            let builder = if config.reminders.sound {
                builder.sound("Default")
            } else {
                builder
            };
            if builder.show().is_err() {
                delivered = false;
            }
        }
        if delivered
            && candidate
                .save(&data_directory().join("reminders.json"))
                .is_ok()
        {
            *reminder_state = candidate;
            if !messages.is_empty() {
                let _ = state.journal.record(diagnostics::EventCode::ReminderSent);
                let _ = app.emit("quota-reminders", &messages);
            }
        }
    }
    if let Some(tray) = app.tray_by_id("quota-tray") {
        let icon = Image::new_owned(
            tray::render(&snapshot),
            tray::ICON_SIZE as u32,
            tray::ICON_SIZE as u32,
        );
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_tooltip(Some(tray::tooltip(&snapshot)));
    }
    if let Some(labels) = app.try_state::<TrayLabels>() {
        let label = |used| {
            tray::remaining(used)
                .map(|v| format!("{v:.0}%"))
                .unwrap_or_else(|| "暂无数据".into())
        };
        let _ = labels.five.set_text(format!(
            "5h：{}",
            label(snapshot.five_hour.as_ref().and_then(|w| w.used_percent))
        ));
        let _ = labels.seven.set_text(format!(
            "7d：{}",
            label(snapshot.seven_day.as_ref().and_then(|w| w.used_percent))
        ));
    }
    Ok(snapshot)
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn cancel_refresh(state: tauri::State<'_, State>) {
    state.refresh.cancel();
}

#[tauri::command]
fn cached_quota(state: tauri::State<'_, State>) -> Option<QuotaSnapshot> {
    state.refresh.cached()
}

#[tauri::command]
fn discover_local_auth() -> Vec<accounts::LocalAuth> {
    accounts::discover_local_auth()
}

#[tauri::command]
fn preview_import(
    path: String,
    alias: String,
    state: tauri::State<'_, State>,
) -> Result<accounts::ImportPreview, String> {
    let names = state
        .refresh
        .cached()
        .map(|snapshot| {
            snapshot
                .accounts
                .into_iter()
                .map(|account| account.alias)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    state
        .imports
        .preview(&accounts::import_path(&path), alias.trim(), &names)
}

#[tauri::command]
async fn account_action(
    app: tauri::AppHandle,
    state: tauri::State<'_, State>,
    action: accounts::AccountAction,
    confirmed: bool,
) -> Result<QuotaSnapshot, String> {
    if !confirmed {
        return Err("账号操作需要先确认".into());
    }
    {
        let _exclusive = state.refresh.exclusive().await;
        let settings = state.settings.lock().map_err(|_| "设置不可用")?.get()?;
        let arguments = accounts::arguments(action, &state.imports)?;
        accounts::execute(&settings, &arguments).await?;
        let _ = state.journal.record(diagnostics::EventCode::AccountChanged);
        state.refresh.invalidate()?;
    }
    refresh_application(&app, &state).await
}

#[tauri::command]
fn deleted_accounts() -> Result<Vec<accounts::DeletedAccount>, String> {
    accounts::deleted_accounts(&accounts::storage_home()?)
}

#[tauri::command]
fn preview_restore(
    id: String,
    alias: String,
    state: tauri::State<'_, State>,
) -> Result<accounts::ImportPreview, String> {
    let path = accounts::archived_auth(&accounts::storage_home()?, &id)?;
    let names = state
        .refresh
        .cached()
        .map(|snapshot| {
            snapshot
                .accounts
                .into_iter()
                .map(|account| account.alias)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    state.imports.preview(&path, &alias, &names)
}

fn create_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "打开主界面", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "立即刷新额度", true, None::<&str>)?;
    let five = MenuItem::with_id(app, "quota-5h", "5h：暂无数据", false, None::<&str>)?;
    let seven = MenuItem::with_id(app, "quota-7d", "7d：暂无数据", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let toggle = MenuItem::with_id(
        app,
        "toggle-overlay",
        "显示/隐藏额度组件",
        true,
        None::<&str>,
    )?;
    let accounts = MenuItem::with_id(app, "accounts", "切换账号…", true, None::<&str>)?;
    let style = MenuItem::with_id(app, "style", "编辑样式…", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &refresh,
            &toggle,
            &accounts,
            &style,
            &separator,
            &five,
            &seven,
            &separator2,
            &quit,
        ],
    )?;
    let initial = QuotaSnapshot::error(QuotaErrorCode::ProviderNotConfigured, "尚未查询", false);
    TrayIconBuilder::with_id("quota-tray")
        .icon(Image::new_owned(
            tray::render(&initial),
            tray::ICON_SIZE as u32,
            tray::ICON_SIZE as u32,
        ))
        .tooltip(tray::tooltip(&initial))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                show_main(app);
            }
            "refresh" => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = refresh_application(&handle, &handle.state::<State>()).await;
                });
            }
            "quit" => request_quit(app),
            "toggle-overlay" => {
                let settings = app
                    .state::<State>()
                    .settings
                    .lock()
                    .ok()
                    .and_then(|store| store.get().ok());
                if let Some(mut settings) = settings {
                    settings.overlay_visible = !settings.overlay_visible;
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(message) =
                            save_settings(handle.clone(), settings, handle.state::<State>()).await
                        {
                            let _ = handle.emit("application-error", message);
                        }
                    });
                }
            }
            "accounts" => {
                show_main(app);
                let _ = app.emit("navigate", "账号管理");
            }
            "style" => {
                show_main(app);
                let _ = app.emit("navigate", "任务栏显示");
            }
            _ => {}
        })
        .build(app)?;
    app.manage(TrayLabels { five, seven, menu });
    Ok(())
}

#[tauri::command]
async fn set_overlay_visible(app: tauri::AppHandle, visible: bool) -> Result<(), String> {
    let state = app.state::<State>();
    let _guard = state.overlay_gate.lock().await;
    update_overlay(&app, visible)
}

fn update_overlay(app: &tauri::AppHandle, visible: bool) -> Result<(), String> {
    if visible {
        if app.get_webview_window("overlay").is_none() {
            WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index.html#overlay".into()))
                .title("CodexQuota Overlay")
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .focused(false)
                .focusable(false)
                .visible(false)
                .resizable(false)
                .inner_size(220.0, 42.0)
                .build()
                .map_err(|e| e.to_string())?;
        }
        let settings = app
            .state::<State>()
            .settings
            .lock()
            .map_err(|_| "设置不可用")?
            .get()?;
        taskbar::sync(app, &settings)?;
        let window = app.get_webview_window("overlay").ok_or("无法创建覆盖层")?;
        taskbar::show_without_activation(&window)?;
    } else if let Some(window) = app.get_webview_window("overlay") {
        taskbar::hide(&window)?;
    }
    Ok(())
}

#[tauri::command]
async fn overlay_menu(app: tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("overlay").ok_or("组件未显示")?;
    window
        .popup_menu(&app.state::<TrayLabels>().menu)
        .map_err(|_| "组件菜单无法打开".into())
}

#[tauri::command]
fn test_notification(app: tauri::AppHandle) -> Result<(), String> {
    notification_setup::register_windows_app()?;
    app.notification()
        .builder()
        .title("CodexQuota 自检")
        .body("Windows 通知通道测试。")
        .show()
        .map_err(|_| "系统通知未能提交".into())
}

#[tauri::command]
fn export_settings(state: tauri::State<'_, State>, path: String) -> Result<String, String> {
    let settings = state.settings.lock().map_err(|_| "设置不可用")?.get()?;
    config_transfer::export(std::path::Path::new(&path), &settings)?;
    Ok("设置已导出，不含认证凭据".into())
}
#[tauri::command]
async fn import_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, State>,
    path: String,
) -> Result<AppSettings, String> {
    let settings = config_transfer::import(std::path::Path::new(&path))?;
    save_settings(app, settings, state).await
}
#[tauri::command]
fn settings_warning(state: tauri::State<'_, State>) -> Option<String> {
    state.settings.lock().ok().and_then(|store| store.warning())
}
#[tauri::command]
fn validate_update_manifest(path: String) -> Result<String, String> {
    let path = std::path::Path::new(&path);
    if !path.is_absolute()
        || !path
            .file_name()
            .and_then(|p| p.to_str())
            .is_some_and(|name| name.ends_with(".codexquota-update.json"))
    {
        return Err("请选择 .codexquota-update.json 清单完整路径".into());
    }
    if std::fs::metadata(path)
        .map_err(|_| "更新清单无法读取")?
        .len()
        > 65536
    {
        return Err("更新清单过大".into());
    }
    let manifest: updates::UpdateManifest =
        serde_json::from_slice(&std::fs::read(path).map_err(|_| "更新清单无法读取")?)
            .map_err(|_| "更新清单结构无效")?;
    manifest.validate(env!("CARGO_PKG_VERSION"))?;
    Ok("仅元数据格式通过。未验证签名真实性；自动下载和安装保持禁用".into())
}
#[tauri::command]
fn export_diagnostics(
    app: tauri::AppHandle,
    state: tauri::State<'_, State>,
    path: String,
) -> Result<String, String> {
    let snapshot = state.refresh.cached();
    let report = diagnostics::DiagnosticReport {
        version: app.package_info().version.to_string(),
        operating_system: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        quota_available: snapshot.as_ref().is_some_and(|s| s.error.is_none()),
        cache_stale: snapshot.as_ref().is_some_and(|s| s.stale),
        account_count: snapshot.as_ref().map_or(0, |s| s.accounts.len()),
        monitor_count: app
            .available_monitors()
            .map_err(|_| "显示器枚举失败")?
            .len(),
        error_code: snapshot
            .as_ref()
            .and_then(|s| s.error.as_ref().map(|e| e.code)),
    };
    diagnostics::export(std::path::Path::new(&path), &report)?;
    Ok("脱敏诊断已导出".into())
}

fn request_quit(app: &tauri::AppHandle) {
    if let Some(state) = app.try_state::<State>() {
        if state
            .exiting
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            return;
        }
        state.refresh.cancel();
        let _ = state.journal.record(diagnostics::EventCode::QuitRequested);
    }
    app.exit(0);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, arguments, _| {
            if arguments.iter().any(|arg| arg == "--quit") {
                request_quit(app);
            } else if !arguments.iter().any(|arg| arg == "--minimized") {
                show_main(app);
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .manage(State {
            journal: diagnostics::Journal::new(data_directory().join("logs")),
            settings: Mutex::new(SettingsStore::load()),
            provider: Mutex::new(QuotaProvider),
            imports: accounts::ImportManager::default(),
            reminders: Mutex::new(reminders::ReminderState::load(
                &data_directory().join("reminders.json"),
            )),
            overlay_ready: std::sync::atomic::AtomicBool::new(false),
            overlay_layout_valid: std::sync::atomic::AtomicBool::new(false),
            overlay_background_matches: std::sync::atomic::AtomicBool::new(false),
            reset_times_visible: std::sync::atomic::AtomicBool::new(false),
            background_cycles: std::sync::atomic::AtomicU64::new(0),
            exiting: std::sync::atomic::AtomicBool::new(false),
            overlay_gate: tokio::sync::Mutex::new(()),
            refresh: refresh::RefreshCoordinator::new(
                std::env::var_os("CODEXQUOTA_DATA_DIR")
                    .or_else(|| std::env::var_os("LOCALAPPDATA"))
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(std::env::temp_dir)
                    .join("CodexQuota")
                    .join("quota-cache.json"),
            ),
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            export_diagnostics,
            export_settings,
            import_settings,
            settings_warning,
            validate_update_manifest,
            autostart::get_autostart,
            autostart::set_autostart,
            save_settings,
            refresh_quota,
            cancel_refresh,
            cached_quota,
            preview_import,
            discover_local_auth,
            account_action,
            deleted_accounts,
            preview_restore,
            taskbar::list_monitors,
            overlay_menu,
            test_notification,
            smoke::is_smoke_test,
            smoke::finish_smoke_test,
            smoke::overlay_loaded,
            set_overlay_visible
        ])
        .setup(|app| {
            if let Err(error) = notification_setup::register_windows_app() {
                eprintln!("CodexQuota Windows notification registration failed: {error}");
            }
            if std::env::args().any(|arg| arg == "--quit") {
                app.handle().exit(0);
                return Ok(());
            }
            create_tray(app)?;
            if std::env::args().any(|arg| arg == "--minimized") {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide()?;
                }
            }
            let _ = app
                .state::<State>()
                .journal
                .record(diagnostics::EventCode::Started);
            let settings = app
                .state::<State>()
                .settings
                .lock()
                .map_err(|_| "设置不可用")?
                .get()?;
            // Setup is safe; later WebView creation must use the async command.
            update_overlay(app.handle(), settings.overlay_visible)?;
            soak::start(app.handle())?;
            let layout_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut last_foreground = taskbar::foreground_window_id();
                let mut last_layout_sync = std::time::Instant::now();
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    let foreground = taskbar::foreground_window_id();
                    let foreground_changed = foreground != last_foreground;
                    last_foreground = foreground;
                    let settings = layout_handle
                        .state::<State>()
                        .settings
                        .lock()
                        .ok()
                        .and_then(|s| s.get().ok());
                    if let Some(settings) = settings {
                        if settings.overlay_visible {
                            if foreground_changed {
                                if let Some(window) = layout_handle.get_webview_window("overlay") {
                                    let hidden = !taskbar::is_visible(&window).unwrap_or(true);
                                    let obstructed =
                                        taskbar::foreground_is_above(&window).unwrap_or(false);
                                    if hidden || obstructed {
                                        let _ = taskbar::reassert_topmost(&window);
                                    }
                                }
                            }
                            if last_layout_sync.elapsed() >= std::time::Duration::from_secs(2) {
                                let _ = taskbar::sync(&layout_handle, &settings);
                                last_layout_sync = std::time::Instant::now();
                            }
                        }
                    }
                }
            });
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let seconds = handle
                        .state::<State>()
                        .settings
                        .lock()
                        .ok()
                        .and_then(|s| s.get().ok())
                        .map(|s| s.refresh_seconds)
                        .unwrap_or(60);
                    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
                    handle
                        .state::<State>()
                        .background_cycles
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let _ = refresh_application(&handle, &handle.state::<State>()).await;
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && !window
                    .state::<State>()
                    .exiting
                    .load(std::sync::atomic::Ordering::SeqCst)
            {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("启动 CodexQuota 失败");
}
