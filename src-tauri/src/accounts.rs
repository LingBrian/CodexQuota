use crate::settings::{validate_cli_path, AppSettings};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

pub fn validate_alias(alias: &str) -> Result<(), String> {
    if alias.is_empty()
        || alias.len() > 64
        || !alias.as_bytes()[0].is_ascii_alphanumeric()
        || !alias
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    {
        return Err("账号别名须为 1–64 个英文字母、数字或 _.-，并以字母或数字开头".into());
    }
    Ok(())
}

#[derive(Default)]
pub struct ImportManager {
    tickets: Mutex<HashMap<String, ImportTicket>>,
}
struct ImportTicket {
    path: PathBuf,
    alias: String,
    digest: String,
    created: Instant,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub ticket_id: String,
    pub alias: String,
    pub size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedAccount {
    pub id: String,
    pub alias: String,
}
pub fn storage_home() -> Result<PathBuf, String> {
    std::env::var_os("CODEX_SWITCH_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE").map(|path| PathBuf::from(path).join(".codex-switch"))
        })
        .ok_or_else(|| "无法定位 codex-switch 数据目录".into())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalAuth {
    pub path: String,
    pub suggested_alias: String,
}

// Only discover known login locations. Never recurse into sessions or return credentials.
pub fn discover_local_auth() -> Vec<LocalAuth> {
    let mut roots = Vec::new();
    if let Some(root) = std::env::var_os("CODEX_HOME").filter(|value| !value.is_empty()) {
        roots.push(PathBuf::from(root));
    }
    if let Some(root) = std::env::var_os("USERPROFILE") {
        roots.push(PathBuf::from(root).join(".codex"));
    }
    discover_in(&roots)
}

fn discover_in(roots: &[PathBuf]) -> Vec<LocalAuth> {
    let mut found = Vec::new();
    let mut seen = Vec::new();
    for root in roots {
        let path = root.join("auth.json");
        if let Ok(canonical) = path.canonicalize() {
            if seen.contains(&canonical) || read_auth(&path).is_err() {
                continue;
            }
            seen.push(canonical);
            found.push(LocalAuth {
                path: path.to_string_lossy().into_owned(),
                suggested_alias: format!("local-codex-{}", found.len() + 1),
            });
        }
    }
    found
}

pub fn import_path(input: &str) -> PathBuf {
    let path = PathBuf::from(input.trim().trim_matches('"'));
    if path.is_dir() {
        path.join("auth.json")
    } else {
        path
    }
}

fn read_auth(path: &Path) -> Result<Vec<u8>, String> {
    let meta = std::fs::symlink_metadata(path).map_err(|_| "无法读取所选认证文件")?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 1024 * 1024 {
        return Err("请选择小于 1 MiB 的普通 JSON 认证文件".into());
    }
    let bytes = std::fs::read(path).map_err(|_| "认证文件读取失败")?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "所选文件不是有效的 JSON")?;
    if !value
        .get("tokens")
        .and_then(|v| v.get("access_token"))
        .and_then(|v| v.as_str())
        .is_some_and(|v| !v.is_empty())
    {
        return Err("所选文件未包含 Codex ChatGPT 登录信息；API Key 认证不能导入为额度账号".into());
    }
    Ok(bytes)
}
pub fn deleted_accounts(root: &Path) -> Result<Vec<DeletedAccount>, String> {
    let deleted = root.join("deleted-profiles");
    if !deleted.exists() {
        return Ok(Vec::new());
    }
    let mut accounts = Vec::new();
    for entry in std::fs::read_dir(&deleted).map_err(|_| "无法读取归档目录")? {
        let entry = entry.map_err(|_| "无法读取归档条目")?;
        if !entry
            .file_type()
            .is_ok_and(|kind| kind.is_dir() && !kind.is_symlink())
        {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        if let Some((alias, timestamp)) = id.rsplit_once(".backup-") {
            if validate_alias(alias).is_ok()
                && !timestamp.is_empty()
                && timestamp.bytes().all(|b| b.is_ascii_digit())
                && entry.path().join("auth.json").is_file()
            {
                accounts.push(DeletedAccount {
                    id: id.clone(),
                    alias: alias.into(),
                });
            }
        }
    }
    accounts.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(accounts)
}
pub fn archived_auth(root: &Path, id: &str) -> Result<PathBuf, String> {
    if !deleted_accounts(root)?
        .iter()
        .any(|account| account.id == id)
    {
        return Err("找不到有效归档，请刷新后再试".into());
    }
    let deleted = root
        .join("deleted-profiles")
        .canonicalize()
        .map_err(|_| "归档目录无法解析")?;
    let path = deleted
        .join(id)
        .join("auth.json")
        .canonicalize()
        .map_err(|_| "归档文件无法解析")?;
    if !path.starts_with(&deleted) {
        return Err("归档路径不在允许的目录中".into());
    }
    Ok(path)
}

impl ImportManager {
    pub fn preview(
        &self,
        path: &Path,
        alias: &str,
        existing: &[String],
    ) -> Result<ImportPreview, String> {
        validate_alias(alias)?;
        if existing.iter().any(|current| current == alias) {
            return Err("该别名已存在，请选择其他名称".into());
        }
        if !path.is_absolute() {
            return Err("请选择认证文件的绝对路径".into());
        }
        let bytes = read_auth(path)?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        let ticket_id = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| "系统时间异常")?
                .as_nanos()
        );
        let mut tickets = self.tickets.lock().map_err(|_| "预览状态不可用")?;
        tickets.retain(|_, ticket| ticket.created.elapsed() < Duration::from_secs(300));
        if tickets.len() >= 8 {
            return Err("导入预览过多，请稍后重试".into());
        }
        tickets.insert(
            ticket_id.clone(),
            ImportTicket {
                path: path.to_owned(),
                alias: alias.into(),
                digest,
                created: Instant::now(),
            },
        );
        Ok(ImportPreview {
            ticket_id,
            alias: alias.into(),
            size_bytes: bytes.len() as u64,
        })
    }
    pub fn consume(&self, id: &str) -> Result<(PathBuf, String), String> {
        let ticket = self
            .tickets
            .lock()
            .map_err(|_| "预览状态不可用")?
            .remove(id)
            .ok_or("预览已失效，请重新选择文件")?;
        if ticket.created.elapsed() >= Duration::from_secs(300) {
            return Err("预览已过期，请重新预览".into());
        }
        let bytes = std::fs::read(&ticket.path).map_err(|_| "认证文件已不可读取")?;
        if format!("{:x}", Sha256::digest(&bytes)) != ticket.digest {
            return Err("文件已在预览后更改，请重新预览".into());
        }
        Ok((ticket.path, ticket.alias))
    }
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AccountAction {
    Switch { alias: String },
    Rename { alias: String, new_alias: String },
    Delete { alias: String },
    Import { ticket_id: String },
    SelectBest,
}

pub async fn execute(settings: &AppSettings, arguments: &[String]) -> Result<(), String> {
    let executable = settings.cli_command();
    validate_cli_path(executable)?;
    let mut command = tokio::process::Command::new(crate::settings::resolve_cli(executable));
    command
        .arg("--json")
        .args(arguments)
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let output = tokio::time::timeout(
        Duration::from_secs(120),
        crate::process::output(&mut command, 4 * 1024 * 1024),
    )
    .await
    .map_err(|_| "账号操作超时。请刷新列表确认实际结果，再决定是否重试")?
    .map_err(|_| "无法启动 codex-switch")?;
    let result: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| "账号操作结果无法解析，请刷新列表确认")?;
    if output.status.success() && result.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(())
    } else {
        Err(action_error(&result))
    }
}

fn action_error(result: &serde_json::Value) -> String {
    // Classify upstream failures without echoing paths, tokens or raw server responses.
    let error = result
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    let detail = if error.contains("token_rotation_lost") {
        "凭据轮换后保存失败，请重新登录 Codex；不要重复导入旧文件"
    } else if error.contains("token_rotated") {
        "凭据已轮换并进入恢复存储，请刷新账号列表检查；不要重复导入旧文件"
    } else if error.contains("cli_auth_credentials_store") || error.contains("keyring") {
        "当前 Codex 使用非文件凭据存储。上游仅支持文件认证，请使用文件模式登录后再导入"
    } else if error.contains("usage_validation") {
        "登录或额度接口验证失败，请检查网络连接及 Codex 登录状态"
    } else if error.contains("structure") || error.contains("file_format") {
        "认证文件结构不完整，请选择 Codex 登录生成的完整 auth.json"
    } else if error.contains("alias") || error.contains("already exists") {
        "账号别名冲突或无效，请更换别名后重新预览"
    } else if error.contains("managed_policy") || error.contains("policy") {
        "账号被本机组织策略限制，请检查允许的登录方式和工作区"
    } else if error.contains("save") || error.contains("permission") || error.contains("denied") {
        "无法保存账号，请检查账号数据目录的写入权限"
    } else {
        "请检查登录状态、网络及账号数据目录权限后重新预览"
    };
    format!("账号操作未完成：{detail}")
}

pub fn arguments(action: AccountAction, manager: &ImportManager) -> Result<Vec<String>, String> {
    match action {
        AccountAction::Switch { alias } => {
            validate_alias(&alias)?;
            Ok(vec!["use".into(), alias])
        }
        AccountAction::SelectBest => Ok(vec!["use".into()]),
        AccountAction::Rename { alias, new_alias } => {
            validate_alias(&alias)?;
            validate_alias(&new_alias)?;
            Ok(vec!["rename".into(), alias, new_alias])
        }
        AccountAction::Delete { alias } => {
            validate_alias(&alias)?;
            Ok(vec!["delete".into(), alias, "--yes".into()])
        }
        AccountAction::Import { ticket_id } => {
            let (path, alias) = manager.consume(&ticket_id)?;
            Ok(vec![
                "import".into(),
                path.to_string_lossy().into_owned(),
                alias,
            ])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovery_is_read_only_deduplicated_and_does_not_expose_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let bytes = br#"{"tokens":{"access_token":"synthetic-secret"}}"#;
        std::fs::write(&path, bytes).unwrap();
        let found = discover_in(&[dir.path().into(), dir.path().into()]);
        assert_eq!(found.len(), 1);
        assert!(!serde_json::to_string(&found)
            .unwrap()
            .contains("synthetic-secret"));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(
            import_path(&format!("  \"{}\"  ", dir.path().display())),
            path
        );
        std::fs::write(&path, b"{}").unwrap();
        assert!(discover_in(&[dir.path().into()]).is_empty());
    }
    #[test]
    fn import_failure_is_actionable_without_leaking_raw_response() {
        let result = serde_json::json!({"ok":false,"error":"usage_validation: secret-token"});
        let message = action_error(&result);
        assert!(message.contains("网络"));
        assert!(!message.contains("secret-token"));
        assert!(
            action_error(&serde_json::json!({"error":"token_rotated: secret"}))
                .contains("不要重复导入")
        );
    }
    #[test]
    fn untrusted_alias_cannot_become_a_flag_or_path() {
        for alias in ["--help", "../work", "work/other", "", "a;b", "a b"] {
            assert!(validate_alias(alias).is_err());
        }
        assert!(validate_alias("work-2").is_ok());
    }
    #[test]
    fn preview_is_nonmutating_and_rejects_file_changes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("auth.json");
        let content = br#"{"tokens":{"access_token":"fake-test-only"}}"#;
        std::fs::write(&path, content).unwrap();
        let manager = ImportManager::default();
        let preview = manager.preview(&path, "work", &[]).unwrap();
        assert!(!serde_json::to_string(&preview)
            .unwrap()
            .contains("fake-test-only"));
        assert_eq!(std::fs::read(&path).unwrap(), content);
        std::fs::write(&path, b"{}").unwrap();
        assert!(manager.consume(&preview.ticket_id).is_err());
    }
    #[test]
    fn existing_alias_is_never_overwritten_by_import_preview() {
        let manager = ImportManager::default();
        assert!(manager
            .preview(Path::new("C:/invalid/auth.json"), "work", &["work".into()])
            .is_err());
    }
}
