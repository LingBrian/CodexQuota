use serde::Serialize;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventCode {
    Started,
    QuitRequested,
    QuotaSucceeded,
    QuotaFailed,
    SettingsSaved,
    AccountChanged,
    ReminderSent,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    pub version: String,
    pub operating_system: String,
    pub architecture: String,
    pub quota_available: bool,
    pub cache_stale: bool,
    pub account_count: usize,
    pub monitor_count: usize,
    pub error_code: Option<crate::quota::QuotaErrorCode>,
}

pub struct Journal {
    directory: PathBuf,
    gate: std::sync::Mutex<()>,
}
impl Journal {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            gate: std::sync::Mutex::new(()),
        }
    }
    pub fn record(&self, code: EventCode) -> Result<(), String> {
        let _guard = self.gate.lock().map_err(|_| "日志锁不可用")?;
        fs::create_dir_all(&self.directory).map_err(|_| "日志目录不可用")?;
        let path = self.directory.join("events.jsonl");
        if fs::metadata(&path).is_ok_and(|metadata| metadata.len() > 64 * 1024) {
            for index in (1..=3).rev() {
                let source = if index == 1 {
                    path.clone()
                } else {
                    self.directory.join(format!("events.{}.jsonl", index - 1))
                };
                let destination = self.directory.join(format!("events.{index}.jsonl"));
                if source.exists() {
                    fs::rename(source, destination).map_err(|_| "日志轮转失败")?;
                }
            }
        }
        let line = serde_json::to_vec(
            &serde_json::json!({"at":chrono::Utc::now().to_rfc3339(),"event":code}),
        )
        .map_err(|_| "日志编码失败")?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| "日志写入失败")?;
        file.write_all(&line)
            .and_then(|_| file.write_all(b"\n"))
            .map_err(|_| "日志写入失败".into())
    }
}
pub fn export(path: &Path, report: &DiagnosticReport) -> Result<(), String> {
    if !path.is_absolute()
        || !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".codexquota-diagnostics.json"))
    {
        return Err("诊断导出路径须以 .codexquota-diagnostics.json 结尾".into());
    }
    if path.exists() {
        return Err("目标文件已存在，请选择新的文件名".into());
    }
    let bytes = serde_json::to_vec_pretty(report).map_err(|_| "诊断报告编码失败")?;
    let parent = path.parent().ok_or("导出路径缺少父目录")?;
    fs::create_dir_all(parent).map_err(|_| "导出目录不可用")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|_| "无法创建导出文件")?;
    temporary
        .write_all(&bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|_| "诊断写入失败")?;
    temporary
        .persist_noclobber(path)
        .map_err(|_| "目标已存在或无法完成导出")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn journal_only_accepts_fixed_codes_and_rotates() {
        let root = tempfile::tempdir().unwrap();
        let journal = Journal::new(root.path().to_owned());
        fs::write(root.path().join("events.jsonl"), vec![b' '; 65537]).unwrap();
        journal.record(EventCode::QuotaFailed).unwrap();
        assert!(root.path().join("events.1.jsonl").exists());
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(root.path().join("events.jsonl")).unwrap()).unwrap();
        assert_eq!(value["event"], "quotaFailed");
        assert_eq!(value.as_object().unwrap().len(), 2);
    }
    #[test]
    fn export_refuses_overwrite_and_unrelated_file_names() {
        let report = DiagnosticReport {
            version: "0.0.0".into(),
            operating_system: "windows".into(),
            architecture: "x86_64".into(),
            quota_available: false,
            cache_stale: false,
            account_count: 0,
            monitor_count: 1,
            error_code: None,
        };
        let root = tempfile::tempdir().unwrap();
        assert!(export(&root.path().join("auth.json"), &report).is_err());
        let path = root.path().join("test.codexquota-diagnostics.json");
        export(&path, &report).unwrap();
        assert!(export(&path, &report).is_err());
    }
}
