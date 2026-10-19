use tauri::Manager;

/// Only a deliberately isolated test process may self-terminate on a timer.
pub fn start(app: &tauri::AppHandle) -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().collect();
    if !arguments.iter().any(|arg| arg == "--soak-test") {
        return Ok(());
    }
    if std::env::var_os("CODEXQUOTA_DATA_DIR").is_none() {
        return Err("长稳测试必须使用独立数据目录".into());
    }
    let seconds = arguments
        .iter()
        .find_map(|arg| arg.strip_prefix("--soak-duration-seconds="))
        .unwrap_or("86400")
        .parse::<u64>()
        .map_err(|_| "长稳测试时长无效")?;
    if !(60..=172800).contains(&seconds) {
        return Err("长稳测试时长必须为 60 至 172800 秒".into());
    }
    crate::update_overlay(app, true)?;
    // Keep positioning recovery enabled during the test without persisting
    // a preference into the user's normal settings directory.
    let state = app.state::<crate::State>();
    let mut settings = state.settings.lock().map_err(|_| "设置不可用")?.get()?;
    settings.overlay_visible = true;
    state
        .settings
        .lock()
        .map_err(|_| "设置不可用")?
        .save(settings)?;
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        handle.state::<crate::State>().refresh.cancel();
        handle.exit(0);
    });
    Ok(())
}
