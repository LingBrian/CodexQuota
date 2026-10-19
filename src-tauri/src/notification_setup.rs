#[cfg(windows)]
pub fn register_windows_app() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, BSTR, PCWSTR};
    use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
    use windows::Win32::System::Com::StructuredStorage::{
        PropVariantClear, PROPVARIANT, PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, STGM_READWRITE,
    };
    use windows::Win32::System::Variant::VT_BSTR;
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    const APP_ID: &str = "io.codexquota.desktop";
    let exe = std::env::current_exe().map_err(|_| "无法定位当前程序路径")?;
    let app_data = std::env::var_os("APPDATA").ok_or("无法定位 Windows 开始菜单目录")?;
    let programs = std::path::PathBuf::from(app_data)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs");
    std::fs::create_dir_all(&programs).map_err(|_| "无法创建 Windows 开始菜单目录")?;
    // Keep portable registration separate from any installer-owned shortcut.
    let shortcut = programs.join("CodexQuota Portable.lnk");
    let wide = |value: &std::ffi::OsStr| {
        value
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>()
    };
    let exe_wide = wide(exe.as_os_str());
    let shortcut_wide = wide(shortcut.as_os_str());
    let description = wide(std::ffi::OsStr::new("CodexQuota 桌面额度监控"));

    // SAFETY: This function creates/updates only this app's per-user Start Menu
    // shortcut. The COM objects and all pointers remain valid for each call.
    unsafe {
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        // RPC_E_CHANGED_MODE means this thread already has a usable COM apartment.
        if initialized.is_err() && initialized.0 != 0x80010106u32 as i32 {
            return Err("Windows 通知组件初始化失败".into());
        }
        let should_uninitialize = initialized.is_ok();
        let result = (|| -> Result<(), String> {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .map_err(|_| "无法创建 Windows 通知应用快捷方式")?;
            let persist: IPersistFile = link
                .cast()
                .map_err(|_| "无法保存 Windows 通知应用快捷方式")?;
            if shortcut.exists() {
                persist
                    .Load(PCWSTR(shortcut_wide.as_ptr()), STGM_READWRITE)
                    .map_err(|_| "无法更新 Windows 通知应用快捷方式")?;
            }
            link.SetPath(PCWSTR(exe_wide.as_ptr()))
                .map_err(|_| "无法设置 Windows 通知应用路径")?;
            link.SetDescription(PCWSTR(description.as_ptr()))
                .map_err(|_| "无法设置 Windows 通知应用名称")?;

            let properties: IPropertyStore =
                link.cast().map_err(|_| "Windows 通知应用身份不可用")?;
            let mut property = PROPVARIANT {
                Anonymous: PROPVARIANT_0 {
                    Anonymous: std::mem::ManuallyDrop::new(PROPVARIANT_0_0 {
                        vt: VT_BSTR,
                        wReserved1: 0,
                        wReserved2: 0,
                        wReserved3: 0,
                        Anonymous: PROPVARIANT_0_0_0 {
                            bstrVal: std::mem::ManuallyDrop::new(BSTR::from(APP_ID)),
                        },
                    }),
                },
            };
            let property_result = properties
                .SetValue(&PKEY_AppUserModel_ID, &property)
                .and_then(|_| properties.Commit());
            let _ = PropVariantClear(&mut property);
            property_result.map_err(|_| "无法注册 Windows 通知应用身份")?;
            persist
                .Save(PCWSTR(shortcut_wide.as_ptr()), true)
                .map_err(|_| "无法写入 Windows 通知快捷方式".to_string())
        })();
        if should_uninitialize {
            CoUninitialize();
        }
        result
    }
}

#[cfg(not(windows))]
pub fn register_windows_app() -> Result<(), String> {
    Ok(())
}
