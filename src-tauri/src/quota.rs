use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{process::Command, time::timeout};

#[derive(Clone, Default)]
pub struct QuotaProvider;
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuotaErrorCode {
    ProviderNotConfigured,
    ProtocolUnverified,
    Internal,
    Timeout,
    Cancelled,
    RefreshInProgress,
    RateLimited,
    CacheWrite,
    NoAccounts,
    ProviderFailed,
    InvalidResponse,
    ProviderUnavailable,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaError {
    pub code: QuotaErrorCode,
    pub message: String,
    pub retryable: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowQuota {
    pub label: String,
    pub used_percent: Option<f64>,
    pub reset_at: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSnapshot {
    pub account: Option<String>,
    pub five_hour: Option<WindowQuota>,
    pub seven_day: Option<WindowQuota>,
    pub updated_at: Option<String>,
    pub source: String,
    pub stale: bool,
    pub error: Option<QuotaError>,
    #[serde(default)]
    pub accounts: Vec<AccountSummary>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub alias: String,
    pub current: bool,
    pub five_hour: Option<WindowQuota>,
    pub seven_day: Option<WindowQuota>,
    pub updated_at: Option<String>,
    pub available: bool,
}
impl QuotaSnapshot {
    pub fn error(code: QuotaErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            account: None,
            five_hour: None,
            seven_day: None,
            updated_at: None,
            source: "未配置的 CLI 适配器".into(),
            stale: false,
            error: Some(QuotaError {
                code,
                message: message.into(),
                retryable,
            }),
            accounts: Vec::new(),
        }
    }
}
impl QuotaProvider {
    pub async fn refresh(&self, settings: &AppSettings) -> QuotaSnapshot {
        let executable = settings.cli_command();
        if crate::settings::validate_cli_path(executable).is_err() {
            return QuotaSnapshot::error(
                QuotaErrorCode::ProviderNotConfigured,
                "未配置经验证的 codex-switch JSON 适配器；不会猜测命令参数或显示模拟额度",
                false,
            );
        }
        let mut command = Command::new(crate::settings::resolve_cli(executable));
        command
            .args(["--json", "list"])
            .kill_on_drop(true)
            .stdin(std::process::Stdio::null());
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        match timeout(
            Duration::from_secs(30),
            crate::process::output(&mut command, 4 * 1024 * 1024),
        )
        .await
        {
            Err(_) => QuotaSnapshot::error(QuotaErrorCode::Timeout, "CLI 查询超时", true),
            Ok(Err(error)) if error.kind() == std::io::ErrorKind::InvalidData => {
                QuotaSnapshot::error(
                    QuotaErrorCode::InvalidResponse,
                    "CLI 输出超出安全上限",
                    false,
                )
            }
            Ok(Err(_)) => QuotaSnapshot::error(
                QuotaErrorCode::ProviderUnavailable,
                "找不到 codex-switch，请安装并加入 PATH",
                false,
            ),
            Ok(Ok(output)) if output.status.success() && output.stdout.len() <= 4 * 1024 * 1024 => {
                crate::adapter::parse(&output.stdout)
            }
            _ => QuotaSnapshot::error(
                QuotaErrorCode::ProviderFailed,
                "codex-switch 查询失败，请检查登录与网络状态",
                true,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "requires explicit local CLI path and a logged-in account"]
    async fn live_provider_returns_real_quota() {
        let executable = std::env::var("CODEXQUOTA_TEST_CLI").expect("set explicit test CLI path");
        let settings = AppSettings {
            quota_command: Some(executable),
            ..AppSettings::default()
        };
        let snapshot = QuotaProvider.refresh(&settings).await;
        assert!(snapshot.error.is_none(), "real provider query must succeed");
        assert!(snapshot.updated_at.is_some());
        assert!(snapshot
            .five_hour
            .as_ref()
            .is_some_and(|w| w.used_percent.is_some()));
        assert!(snapshot
            .seven_day
            .as_ref()
            .is_some_and(|w| w.used_percent.is_some()));
    }
    #[test]
    fn error_snapshot_never_invents_quota() {
        let result = QuotaSnapshot::error(QuotaErrorCode::Internal, "test", false);
        assert!(result.five_hour.is_none());
        assert!(result.seven_day.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn error_code_is_machine_readable() {
        let result = QuotaSnapshot::error(QuotaErrorCode::ProtocolUnverified, "协议未验证", false);
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["error"]["code"], "protocolUnverified");
    }
}
