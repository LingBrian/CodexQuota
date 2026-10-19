use crate::{settings::atomic_write, State};
use serde::Serialize;
use tauri::Manager;

#[tauri::command]
pub fn is_smoke_test() -> bool {
    std::env::args().any(|arg| arg == "--smoke-test")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeReport {
    version: String,
    webview_ipc: bool,
    main_window: bool,
    tray_initialized: bool,
    progress_bars: u8,
    has_five_hour: bool,
    has_seven_day: bool,
    query_succeeded: bool,
    passed: bool,
    import_preview: bool,
    overlay_webview: bool,
    overlay_unfocused: bool,
    account_rows: u32,
    monitors: usize,
    overlay_in_work_area: bool,
    appbar_detected: bool,
    background_cycles: u64,
    notification_submitted: Option<bool>,
    overlay_geometry: serde_json::Value,
    overlay_recovered: bool,
    interaction_verified: bool,
    interaction_states: serde_json::Value,
    presentation_verified: bool,
    diagnostic_exported: bool,
    overlay_layout_valid: bool,
    overlay_in_taskbar: bool,
    visibility_verified: bool,
    position_settings_applied: bool,
    background_matches: bool,
    reset_times_visible: bool,
    overlay_topmost: bool,
}

#[tauri::command]
pub fn overlay_loaded(
    state: tauri::State<'_, State>,
    layout_valid: bool,
    background_matches: bool,
    reset_times_visible: bool,
) {
    state
        .overlay_layout_valid
        .store(layout_valid, std::sync::atomic::Ordering::SeqCst);
    state
        .overlay_ready
        .store(true, std::sync::atomic::Ordering::SeqCst);
    state
        .overlay_background_matches
        .store(background_matches, std::sync::atomic::Ordering::SeqCst);
    state
        .reset_times_visible
        .store(reset_times_visible, std::sync::atomic::Ordering::SeqCst);
}

#[tauri::command]
pub async fn finish_smoke_test(
    app: tauri::AppHandle,
    state: tauri::State<'_, State>,
    progress_bars: u8,
    account_rows: u32,
    presentation_verified: bool,
) -> Result<(), String> {
    if !is_smoke_test() {
        return Err("自检模式未启用".into());
    }
    let root = std::env::var_os("CODEXQUOTA_DATA_DIR").ok_or("自检必须设置独立数据目录")?;
    let diagnostic_path =
        std::path::PathBuf::from(&root).join("runtime.codexquota-diagnostics.json");
    crate::export_diagnostics(
        app.clone(),
        app.state::<State>(),
        diagnostic_path.to_string_lossy().into_owned(),
    )?;
    let diagnostic_exported = std::fs::read(&diagnostic_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .is_some_and(|value| value.as_object().is_some_and(|object| object.len() == 8));
    let snapshot = state.refresh.cached();
    let query_succeeded = snapshot.as_ref().is_some_and(|s| s.error.is_none());
    let main_window = app.get_webview_window("main").is_some();
    let tray_initialized = app.tray_by_id("quota-tray").is_some();
    let has_five_hour = snapshot.as_ref().is_some_and(|s| {
        s.five_hour
            .as_ref()
            .is_some_and(|w| w.used_percent.is_some())
    });
    let has_seven_day = snapshot.as_ref().is_some_and(|s| {
        s.seven_day
            .as_ref()
            .is_some_and(|w| w.used_percent.is_some())
    });
    let directory = tempfile::tempdir().map_err(|_| "无法创建预览测试目录")?;
    let credential_path = directory.path().join("auth.json");
    std::fs::write(
        &credential_path,
        br#"{"tokens":{"access_token":"synthetic-preview-only"}}"#,
    )
    .map_err(|_| "无法准备预览测试")?;
    let preview = state
        .imports
        .preview(&credential_path, "runtime-preview", &[])?;
    let import_preview = state.imports.consume(&preview.ticket_id).is_ok();
    crate::set_overlay_visible(app.clone(), true).await?;
    let window = app.get_webview_window("overlay").ok_or("组件不存在")?;
    let initially_visible = crate::taskbar::is_visible(&window)?;
    crate::set_overlay_visible(app.clone(), false).await?;
    let actually_hidden = !crate::taskbar::is_visible(&window)?;
    crate::set_overlay_visible(app.clone(), true).await?;
    let visibility_verified =
        initially_visible && actually_hidden && crate::taskbar::is_visible(&window)?;
    let original_position = window.outer_position().map_err(|_| "无法读取组件位置")?;
    let mut original = state.settings.lock().map_err(|_| "设置不可用")?.get()?;
    original.overlay_visible = true;
    let mut moved = original.clone();
    moved.overlay.edge_offset += 120;
    moved.overlay.offset_x += 120;
    crate::save_settings(app.clone(), moved, app.state::<State>()).await?;
    let position_settings_applied = window
        .outer_position()
        .is_ok_and(|position| position != original_position);
    crate::save_settings(app.clone(), original, app.state::<State>()).await?;
    let overlay_webview = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while !state
            .overlay_ready
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
    .await
    .is_ok();
    let mut overlay_recovered = false;
    if let Some(window) = app.get_webview_window("overlay") {
        let original = window.outer_position().map_err(|_| "无法读取组件坐标")?;
        window
            .set_position(tauri::PhysicalPosition::new(-30000, -30000))
            .map_err(|_| "恢复测试无法移动组件")?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let mut config = state.settings.lock().map_err(|_| "设置不可用")?.get()?;
        config.overlay.monitor = Some(32); // Valid saved index, disconnected here.
        crate::taskbar::sync(&app, &config)?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        overlay_recovered = window
            .outer_position()
            .is_ok_and(|position| position == original);
    }
    let overlay_unfocused = app
        .get_webview_window("overlay")
        .is_some_and(|window| !window.is_focused().unwrap_or(true));
    let overlay_layout_valid = state
        .overlay_layout_valid
        .load(std::sync::atomic::Ordering::SeqCst);
    let background_matches = state
        .overlay_background_matches
        .load(std::sync::atomic::Ordering::SeqCst);
    let reset_times_visible = state
        .reset_times_visible
        .load(std::sync::atomic::Ordering::SeqCst);
    let mut interaction_verified = false;
    let mut interaction_states = serde_json::Value::Null;
    #[cfg(windows)]
    if let Some(window) = app.get_webview_window("overlay") {
        let mut config = state.settings.lock().map_err(|_| "设置不可用")?.get()?;
        config.overlay.click_through = true;
        crate::save_settings(app.clone(), config.clone(), app.state::<State>()).await?;
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let enabled = crate::taskbar::interaction_state(&window)?;
        config.overlay.click_through = false;
        crate::save_settings(app.clone(), config, app.state::<State>()).await?;
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let disabled = crate::taskbar::interaction_state(&window)?;
        interaction_verified = enabled.click_through
            && !disabled.click_through
            && enabled.no_activate
            && disabled.no_activate
            && enabled.tool_window
            && disabled.tool_window;
        interaction_states = serde_json::json!({"enabled":enabled,"disabled":disabled});
    }
    let monitors = app
        .available_monitors()
        .map_err(|_| "显示器枚举失败")?
        .len();
    let overlay_in_work_area = if let Some(window) = app.get_webview_window("overlay") {
        if let (Ok(position), Ok(size), Ok(monitors)) = (
            window.outer_position(),
            window.outer_size(),
            app.available_monitors(),
        ) {
            monitors.iter().any(|monitor| {
                let work = monitor.work_area();
                i64::from(position.x) >= i64::from(work.position.x)
                    && i64::from(position.y) >= i64::from(work.position.y)
                    && i64::from(position.x) + i64::from(size.width)
                        <= i64::from(work.position.x) + i64::from(work.size.width)
                    && i64::from(position.y) + i64::from(size.height)
                        <= i64::from(work.position.y) + i64::from(work.size.height)
            })
        } else {
            false
        }
    } else {
        false
    };
    #[cfg(windows)]
    let appbar_detected = crate::taskbar::primary_taskbar().is_some();
    #[cfg(not(windows))]
    let appbar_detected = false;
    #[cfg(windows)]
    let overlay_in_taskbar = crate::taskbar::primary_taskbar().is_some_and(|bar| {
        let Ok(position) = window.outer_position() else {
            return false;
        };
        let Ok(size) = window.outer_size() else {
            return false;
        };
        position.x >= bar.x
            && position.y >= bar.y
            && position.x as i64 + size.width as i64 <= bar.x as i64 + bar.width as i64
            && position.y as i64 + size.height as i64 <= bar.y as i64 + bar.height as i64
    });
    #[cfg(not(windows))]
    let overlay_in_taskbar = false;
    #[cfg(windows)]
    let overlay_topmost = app
        .get_webview_window("overlay")
        .and_then(|window| crate::taskbar::is_topmost(&window).ok())
        .unwrap_or(false);
    #[cfg(not(windows))]
    let overlay_topmost = true;
    let background_requested = std::env::args().any(|arg| arg == "--test-background");
    let overlay_geometry=app.get_webview_window("overlay").map(|window|serde_json::json!({"position":window.outer_position().ok(),"size":window.outer_size().ok(),"monitors":app.available_monitors().ok().map(|monitors|monitors.into_iter().map(|monitor|serde_json::json!({"position":monitor.position(),"size":monitor.size(),"work":monitor.work_area(),"scale":monitor.scale_factor()})).collect::<Vec<_>>())})).unwrap_or_default();
    if background_requested {
        let _ = tokio::time::timeout(std::time::Duration::from_secs(75), async {
            while state
                .background_cycles
                .load(std::sync::atomic::Ordering::SeqCst)
                == 0
            {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
        .await;
    }
    let background_cycles = state
        .background_cycles
        .load(std::sync::atomic::Ordering::SeqCst);
    let notification_submitted = std::env::args()
        .any(|arg| arg == "--test-notification")
        .then(|| crate::test_notification(app.clone()).is_ok());
    let passed = query_succeeded
        && overlay_layout_valid
        && presentation_verified
        && diagnostic_exported
        && main_window
        && tray_initialized
        && has_five_hour
        && has_seven_day
        && progress_bars == 2
        && import_preview
        && overlay_webview
        && overlay_unfocused
        && account_rows > 0
        && monitors > 0;
    let passed = passed
        && overlay_recovered
        && interaction_verified
        && (overlay_in_taskbar || overlay_in_work_area)
        && visibility_verified
        && position_settings_applied
        && background_matches
        && reset_times_visible
        && appbar_detected
        && overlay_topmost
        && (!background_requested || background_cycles > 0)
        && notification_submitted != Some(false);
    let report = RuntimeReport {
        version: app.package_info().version.to_string(),
        webview_ipc: true,
        main_window,
        tray_initialized,
        progress_bars,
        has_five_hour,
        has_seven_day,
        query_succeeded,
        passed,
        import_preview,
        overlay_webview,
        overlay_unfocused,
        account_rows,
        monitors,
        overlay_in_work_area,
        appbar_detected,
        background_cycles,
        notification_submitted,
        overlay_geometry,
        overlay_recovered,
        interaction_verified,
        interaction_states,
        presentation_verified,
        diagnostic_exported,
        overlay_layout_valid,
        overlay_in_taskbar,
        visibility_verified,
        position_settings_applied,
        background_matches,
        reset_times_visible,
        overlay_topmost,
    };
    let path = std::path::PathBuf::from(root).join("runtime-smoke.json");
    atomic_write(
        &path,
        &serde_json::to_vec_pretty(&report).map_err(|_| "无法序列化自检报告")?,
    )?;
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        if passed {
            crate::request_quit(&app);
        } else {
            app.exit(1);
        }
    });
    Ok(())
}
