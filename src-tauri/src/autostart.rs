fn command_line(executable: &std::path::Path) -> Result<String, String> {
    if !executable.is_absolute() || executable.to_string_lossy().contains(['"', '\r', '\n']) {
        return Err("启动程序路径无效".into());
    }
    Ok(format!("\"{}\" --minimized", executable.display()))
}
#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

#[tauri::command]
pub fn get_autostart() -> Result<bool, String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Registry::{
            RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ,
        };
        let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let name = wide("CodexQuota");
        let mut bytes = 0;
        // SAFETY: both names are NUL terminated; the first call only queries size.
        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut bytes,
            )
        };
        match result {
            0 => Ok(bytes > 2),
            2 => Ok(false),
            _ => Err("无法读取当前用户启动项".into()),
        }
    }
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

/// Called only after an explicit user toggle; never enabled during setup/tests.
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<bool, String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Registry::{
            RegDeleteKeyValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ,
        };
        let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let name = wide("CodexQuota");
        let command = wide(&command_line(
            &std::env::current_exe().map_err(|_| "无法获取程序路径")?,
        )?);
        // SAFETY: bounded UTF-16 buffers stay alive for this synchronous call.
        // Only our named value in the current user's Run key is changed.
        let result = unsafe {
            if enabled {
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    key.as_ptr(),
                    name.as_ptr(),
                    REG_SZ,
                    command.as_ptr().cast(),
                    (command.len() * 2) as u32,
                )
            } else {
                RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr())
            }
        };
        if result != 0 && !(result == 2 && !enabled) {
            return Err("无法更新当前用户启动项".into());
        }
        get_autostart()
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("仅支持 Windows 启动项".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quotes_paths_with_spaces_and_never_invokes_shell() {
        let root = std::env::current_dir()
            .unwrap()
            .join("A B")
            .join("CodexQuota.exe");
        let command = command_line(&root).unwrap();
        assert!(command.starts_with('"'));
        assert!(command.ends_with("\" --minimized"));
        assert!(command_line(std::path::Path::new("relative.exe")).is_err());
    }
}
