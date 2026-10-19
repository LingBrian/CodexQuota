use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppSettings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_accent")]
    pub accent_color: String,
    #[serde(default)]
    pub overlay: crate::taskbar::OverlaySettings,
    #[serde(default)]
    pub reminders: crate::reminders::ReminderSettings,
    #[serde(default = "default_settings_version")]
    pub settings_version: u8,
    pub refresh_seconds: u64,
    pub quota_command: Option<String>,
    pub overlay_visible: bool,
    pub overlay_text_color: String,
    pub overlay_font_size: u8,
    pub overlay_opacity: f32,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: default_language(),
            theme: default_theme(),
            accent_color: default_accent(),
            overlay: crate::taskbar::OverlaySettings::default(),
            reminders: crate::reminders::ReminderSettings::default(),
            settings_version: default_settings_version(),
            refresh_seconds: 60,
            quota_command: Some("codex-switch".into()),
            overlay_visible: false,
            overlay_text_color: "#ffffff".into(),
            overlay_font_size: 14,
            overlay_opacity: 0.92,
        }
    }
}
const fn default_settings_version() -> u8 {
    2
}
fn default_language() -> String {
    "zh-CN".into()
}
fn default_theme() -> String {
    "system".into()
}
fn default_accent() -> String {
    "#75dfbb".into()
}
impl AppSettings {
    pub fn cli_command(&self) -> &str {
        self.quota_command
            .as_deref()
            .filter(|path| !path.trim().is_empty())
            .unwrap_or("codex-switch")
    }

    pub fn validate(&self) -> Result<(), String> {
        if !["zh-CN", "en"].contains(&self.language.as_str())
            || !["system", "light", "dark"].contains(&self.theme.as_str())
            || self.accent_color.len() != 7
            || !self.accent_color.starts_with('#')
            || !self.accent_color[1..]
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err("语言、主题或强调色无效".into());
        }
        self.overlay.validate()?;
        self.reminders.validate()?;
        if self.settings_version != 2 {
            return Err("不支持的设置版本，请先升级应用".into());
        }
        if self.overlay_text_color.len() != 7
            || !self.overlay_text_color.starts_with('#')
            || !self.overlay_text_color[1..]
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err("颜色必须为 #RRGGBB".into());
        }
        if !(60..=3600).contains(&self.refresh_seconds) {
            return Err("刷新间隔必须在 60 到 3600 秒之间".into());
        }
        if !(10..=36).contains(&self.overlay_font_size) {
            return Err("字体大小必须在 10 到 36 之间".into());
        }
        if !(0.2..=1.0).contains(&self.overlay_opacity) {
            return Err("透明度必须在 0.2 到 1 之间".into());
        }
        validate_cli_path(self.cli_command())?;
        Ok(())
    }
}
pub fn validate_cli_path(command: &str) -> Result<(), String> {
    if command == "codex-switch" {
        return Ok(());
    }
    let path = Path::new(command);
    if !path.is_absolute()
        || path.file_name().and_then(|name| name.to_str()) != Some("codex-switch.exe")
    {
        return Err("请选择 codex-switch.exe 的绝对路径，或使用 PATH 中的 codex-switch".into());
    }
    Ok(())
}
pub fn resolve_cli(command: &str) -> PathBuf {
    if command == "codex-switch" {
        if let Ok(executable) = std::env::current_exe() {
            if let Some(directory) = executable.parent() {
                let bundled = directory.join("codex-switch.exe");
                if bundled.is_file() {
                    return bundled;
                }
            }
        }
    }
    PathBuf::from(command)
}
pub struct SettingsStore {
    path: PathBuf,
    value: AppSettings,
    warning: Option<String>,
}
impl SettingsStore {
    pub fn load() -> Self {
        if let Some(directory) = std::env::var_os("CODEXQUOTA_DATA_DIR") {
            return Self::load_at(PathBuf::from(directory).join("settings.json"));
        }
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("CodexQuota");
        let path = base.join("settings.json");
        Self::load_at(path)
    }
    fn load_at(path: PathBuf) -> Self {
        let (value, warning) = match fs::read(&path) {
            Ok(bytes) => match crate::config_transfer::decode(&bytes) {
                Ok(value) => (value, None),
                Err(_) => (
                    AppSettings::default(),
                    Some(
                        "原设置不兼容或损坏；已只读保留，使用临时默认设置。请先备份原设置文件"
                            .into(),
                    ),
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (AppSettings::default(), None)
            }
            Err(_) => (
                AppSettings::default(),
                Some("原设置无法读取，禁止覆盖；请检查文件权限".into()),
            ),
        };
        Self {
            path,
            value,
            warning,
        }
    }
    pub fn warning(&self) -> Option<String> {
        self.warning.clone()
    }
    pub fn get(&self) -> Result<AppSettings, String> {
        Ok(self.value.clone())
    }
    pub fn save(&mut self, value: AppSettings) -> Result<(), String> {
        if let Some(warning) = &self.warning {
            return Err(warning.clone());
        }
        value.validate()?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
        atomic_write(&self.path, content.as_bytes())?;
        self.value = value;
        Ok(())
    }
}

pub fn atomic_write(path: &Path, content: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("文件缺少父目录")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary.write_all(content).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incompatible_settings_remain_untouched() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("settings.json");
        let mut value = serde_json::to_value(AppSettings::default()).unwrap();
        value["settingsVersion"] = serde_json::json!(99);
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(&path, &bytes).unwrap();
        let mut store = SettingsStore::load_at(path.clone());
        assert!(store.warning().is_some());
        assert!(store.save(AppSettings::default()).is_err());
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    #[test]
    fn legacy_settings_migrate_to_current_schema() {
        let legacy = r##"{"refreshSeconds":120,"overlayVisible":true,"overlayTextColor":"#ffffff","overlayFontSize":14,"overlayOpacity":0.9}"##;
        let settings: AppSettings = serde_json::from_str(legacy).unwrap();
        assert_eq!(settings.settings_version, 2);
        assert_eq!(settings.refresh_seconds, 120);
        assert!(settings.quota_command.is_none());
        assert_eq!(settings.cli_command(), "codex-switch");
    }
    #[test]
    fn null_and_empty_cli_settings_use_the_packaged_provider() {
        for command in [None, Some(String::new()), Some(" ".into())] {
            let settings = AppSettings {
                quota_command: command,
                ..AppSettings::default()
            };
            assert_eq!(settings.cli_command(), "codex-switch");
            assert!(settings.validate().is_ok());
        }
    }
    #[test]
    fn unsafe_command_is_rejected() {
        let settings = AppSettings {
            quota_command: Some("cmd.exe /c whoami".into()),
            ..AppSettings::default()
        };
        assert!(settings.validate().is_err());
    }
    #[test]
    fn saved_settings_can_be_read_back() {
        let path =
            std::env::temp_dir().join(format!("codexquota-settings-{}.json", std::process::id()));
        let mut store = SettingsStore {
            path: path.clone(),
            value: AppSettings::default(),
            warning: None,
        };
        let next = AppSettings {
            refresh_seconds: 180,
            ..AppSettings::default()
        };
        store.save(next).unwrap();
        let replacement = AppSettings {
            refresh_seconds: 240,
            ..AppSettings::default()
        };
        store.save(replacement).unwrap();
        let saved: AppSettings = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved.refresh_seconds, 240);
        fs::remove_file(path).unwrap();
    }
}
