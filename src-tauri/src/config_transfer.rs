use crate::settings::AppSettings;
use std::io::Write;
use std::path::Path;

pub fn decode(bytes: &[u8]) -> Result<AppSettings, String> {
    if bytes.len() > 64 * 1024 {
        return Err("设置文件不得超过 64 KiB".into());
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| "设置文件不是有效的 JSON")?;
    if value
        .get("settingsVersion")
        .and_then(|version| version.as_u64())
        == Some(1)
    {
        value["settingsVersion"] = serde_json::json!(2);
    }
    let mut settings: AppSettings = serde_json::from_value(value).map_err(|_| "设置结构不兼容")?;
    if settings.overlay.layout_version == 0 {
        if settings.overlay.width <= 300 {
            settings.overlay.width = 420;
            settings.overlay.height = settings.overlay.height.max(48);
            settings.overlay.gap = settings.overlay.gap.min(12);
        }
        settings.overlay.layout_version = 1;
    }
    settings.validate()?;
    Ok(settings)
}
pub fn import(path: &Path) -> Result<AppSettings, String> {
    if !path.is_absolute()
        || !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".codexquota.json"))
    {
        return Err("请选择 .codexquota.json 设置文件的完整路径".into());
    }
    if std::fs::metadata(path)
        .map_err(|_| "无法读取设置文件")?
        .len()
        > 65536
    {
        return Err("设置文件过大".into());
    }
    decode(&std::fs::read(path).map_err(|_| "设置文件读取失败")?)
}
pub fn export(path: &Path, settings: &AppSettings) -> Result<(), String> {
    settings.validate()?;
    if !path.is_absolute()
        || !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".codexquota.json"))
    {
        return Err("设置导出路径须以 .codexquota.json 结尾".into());
    }
    if path.exists() {
        return Err("目标文件已存在，请使用新的文件名".into());
    }
    let parent = path.parent().ok_or("导出路径缺少父目录")?;
    std::fs::create_dir_all(parent).map_err(|_| "导出目录不可用")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|_| "无法创建导出文件")?;
    temporary
        .write_all(&serde_json::to_vec_pretty(settings).map_err(|_| "设置编码失败")?)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|_| "设置导出失败")?;
    temporary
        .persist_noclobber(path)
        .map_err(|_| "目标已存在或无法导出")?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_preserves_settings_and_migrates_version_one() {
        let settings = AppSettings {
            refresh_seconds: 120,
            ..Default::default()
        };
        let mut value = serde_json::to_value(&settings).unwrap();
        value["settingsVersion"] = serde_json::json!(1);
        let restored = decode(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(restored.settings_version, 2);
        assert_eq!(restored.refresh_seconds, 120);
    }
    #[test]
    fn legacy_compact_overlay_is_upgraded_for_full_reset_times() {
        let mut value = serde_json::to_value(AppSettings::default()).unwrap();
        value["overlay"]["width"] = serde_json::json!(300);
        value["overlay"]["height"] = serde_json::json!(46);
        let overlay = value["overlay"].as_object_mut().unwrap();
        overlay.remove("layoutVersion");
        overlay.remove("displayStyle");
        overlay.remove("cornerRadius");
        let restored = decode(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(restored.overlay.width, 420);
        assert_eq!(restored.overlay.height, 48);
        assert_eq!(restored.overlay.layout_version, 1);
        assert_eq!(restored.overlay.display_style, "bars");
        assert_eq!(restored.overlay.corner_radius, 10);
        assert_eq!(restored.overlay.monitor, None);
    }
    #[test]
    fn future_versions_and_credentials_are_rejected() {
        let mut value = serde_json::to_value(AppSettings::default()).unwrap();
        value["settingsVersion"] = serde_json::json!(99);
        assert!(decode(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(decode(br#"{"tokens":{"access_token":"test"}}"#).is_err());
    }
}
