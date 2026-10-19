//! Protocol verified against xjoker/codex-switch 20261001.5.0, src/output.rs.
use crate::quota::{AccountSummary, QuotaErrorCode, QuotaSnapshot, WindowQuota};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct ListOutput {
    pub profiles: Vec<Profile>,
}
#[derive(Clone, Deserialize)]
pub struct Profile {
    pub alias: String,
    pub is_current: bool,
    pub usage: Usage,
}
#[derive(Clone, Deserialize)]
pub struct Usage {
    pub fetched_at: Option<String>,
    pub primary: Option<Window>,
    pub secondary: Option<Window>,
    pub error: Option<String>,
}
#[derive(Clone, Deserialize)]
pub struct Window {
    pub label: String,
    pub used_percent: Option<f64>,
    pub resets_at: Option<i64>,
}

pub fn window(window: Option<Window>, expected_label: &str) -> Option<WindowQuota> {
    window
        .filter(|w| w.label == expected_label)
        .map(|w| WindowQuota {
            label: w.label,
            used_percent: w
                .used_percent
                .filter(|value| value.is_finite() && (0.0..=100.0).contains(value)),
            reset_at: w
                .resets_at
                .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
                .map(|date| date.to_rfc3339()),
        })
}
pub fn parse(bytes: &[u8]) -> QuotaSnapshot {
    let Ok(output) = serde_json::from_slice::<ListOutput>(bytes) else {
        return QuotaSnapshot::error(
            QuotaErrorCode::InvalidResponse,
            "CLI 返回的数据格式不兼容",
            false,
        );
    };
    let accounts: Vec<AccountSummary> = output
        .profiles
        .iter()
        .map(|profile| AccountSummary {
            alias: profile.alias.clone(),
            current: profile.is_current,
            five_hour: window(profile.usage.primary.clone(), "5h"),
            seven_day: window(profile.usage.secondary.clone(), "7d"),
            updated_at: profile.usage.fetched_at.clone(),
            available: profile.usage.error.is_none(),
        })
        .collect();
    let Some(profile) = output.profiles.into_iter().find(|p| p.is_current) else {
        let mut result = QuotaSnapshot::error(
            QuotaErrorCode::NoAccounts,
            "没有已激活的 Codex 账号，请在账号管理中导入",
            false,
        );
        result.accounts = accounts;
        return result;
    };
    if profile.usage.error.is_some() {
        // Do not forward provider-controlled text: errors may contain URLs or credentials.
        let mut result = QuotaSnapshot::error(
            QuotaErrorCode::ProviderFailed,
            "当前账号额度查询失败，请检查登录与网络状态",
            true,
        );
        result.account = Some(profile.alias);
        result.accounts = accounts;
        return result;
    }
    let updated_at = profile
        .usage
        .fetched_at
        .filter(|text| chrono::DateTime::parse_from_rfc3339(text).is_ok());
    if updated_at.is_none() {
        return QuotaSnapshot::error(
            QuotaErrorCode::InvalidResponse,
            "CLI 未提供可靠的查询时间",
            false,
        );
    }
    QuotaSnapshot {
        account: Some(profile.alias),
        five_hour: window(profile.usage.primary, "5h"),
        seven_day: window(profile.usage.secondary, "7d"),
        updated_at,
        source: "codex-switch --json list".into(),
        stale: false,
        error: None,
        accounts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_source_derived_contract_and_ignores_sensitive_unknown_fields() {
        let output = br#"{"profiles":[{"alias":"work","is_current":true,"tokens":{"access_token":"never-forward"},"usage":{"fetched_at":"2026-10-16T00:00:00Z","primary":{"label":"5h","used_percent":25,"resets_at":1792108800},"secondary":{"label":"7d","used_percent":65,"resets_at":null}}}]}"#;
        let result = parse(output);
        assert_eq!(result.five_hour.as_ref().unwrap().used_percent, Some(25.0));
        assert_eq!(result.seven_day.as_ref().unwrap().used_percent, Some(65.0));
        assert!(!serde_json::to_string(&result)
            .unwrap()
            .contains("never-forward"));
    }
    #[test]
    fn empty_real_cli_output_has_no_fake_quota() {
        let result = parse(br#"{"profiles":[]}"#);
        assert!(result.five_hour.is_none());
        assert!(matches!(
            result.error.unwrap().code,
            QuotaErrorCode::NoAccounts
        ));
    }
    #[test]
    fn missing_percentage_and_unknown_label_are_not_full_quota() {
        assert!(window(
            Some(Window {
                label: "5h".into(),
                used_percent: None,
                resets_at: None
            }),
            "5h"
        )
        .unwrap()
        .used_percent
        .is_none());
        assert!(window(
            Some(Window {
                label: "other".into(),
                used_percent: Some(0.0),
                resets_at: None
            }),
            "5h"
        )
        .is_none());
    }
}
