use crate::{quota::QuotaSnapshot, tray::remaining};
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReminderSettings {
    pub enabled: bool,
    pub five_hour_thresholds: Vec<u8>,
    pub seven_day_thresholds: Vec<u8>,
    pub quiet_start: u8,
    pub quiet_end: u8,
    pub recovery: bool,
    pub query_failure: bool,
    pub reset_soon: bool,
    pub sound: bool,
}
impl Default for ReminderSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            five_hour_thresholds: vec![20, 10, 5],
            seven_day_thresholds: vec![20, 10, 5],
            quiet_start: 22,
            quiet_end: 7,
            recovery: true,
            query_failure: true,
            reset_soon: false,
            sound: false,
        }
    }
}
impl ReminderSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.quiet_start > 23
            || self.quiet_end > 23
            || self.five_hour_thresholds.len() > 10
            || self.seven_day_thresholds.len() > 10
            || self
                .five_hour_thresholds
                .iter()
                .chain(self.seven_day_thresholds.iter())
                .any(|threshold| *threshold == 0 || *threshold > 100)
        {
            Err("请设置 1–100 的额度阈值和 0–23 的免打扰小时".into())
        } else {
            Ok(())
        }
    }
    pub fn quiet(&self, hour: u8) -> bool {
        if self.quiet_start == self.quiet_end {
            false
        } else if self.quiet_start < self.quiet_end {
            (self.quiet_start..self.quiet_end).contains(&hour)
        } else {
            hour >= self.quiet_start || hour < self.quiet_end
        }
    }
}
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ReminderState {
    sent: HashSet<String>,
    previous: HashMap<String, f64>,
}
impl ReminderState {
    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        crate::settings::atomic_write(
            path,
            &serde_json::to_vec(self).map_err(|_| "提醒状态无法序列化")?,
        )
    }
    pub fn evaluate(
        &mut self,
        snapshot: &QuotaSnapshot,
        settings: &ReminderSettings,
        now: DateTime<Utc>,
        local_hour: u8,
    ) -> Vec<String> {
        if !settings.enabled || settings.quiet(local_hour) {
            return Vec::new();
        }
        if snapshot.error.is_some() {
            if settings.query_failure && self.sent.insert("query-failure".into()) {
                return vec!["额度查询失败，请检查网络与登录状态。".into()];
            }
            return Vec::new();
        }
        self.sent.remove("query-failure");
        if snapshot.stale {
            return Vec::new();
        }
        let Some(account) = snapshot.account.as_deref() else {
            return Vec::new();
        };
        let mut messages = Vec::new();
        for (name, window, thresholds) in [
            ("5h", &snapshot.five_hour, &settings.five_hour_thresholds),
            ("7d", &snapshot.seven_day, &settings.seven_day_thresholds),
        ] {
            let Some(window) = window else {
                continue;
            };
            let Some(percent) = remaining(window.used_percent) else {
                continue;
            };
            let reset = window
                .reset_at
                .as_deref()
                .and_then(|text| DateTime::parse_from_rfc3339(text).ok());
            if reset.is_some_and(|time| time <= now) {
                continue;
            }
            let cycle = format!(
                "{account}/{name}/{}",
                window.reset_at.as_deref().unwrap_or("unknown")
            );
            let mut crossed = thresholds
                .iter()
                .copied()
                .filter(|threshold| percent < f64::from(*threshold))
                .collect::<Vec<_>>();
            crossed.sort_unstable();
            if let Some(threshold) = crossed
                .iter()
                .find(|threshold| !self.sent.contains(&format!("{cycle}/{threshold}")))
            {
                messages.push(format!(
                    "{name} 剩余额度 {percent:.0}%，已低于 {threshold}%。"
                ));
                for threshold in crossed {
                    self.sent.insert(format!("{cycle}/{threshold}"));
                }
            }
            let previous = self.previous.insert(format!("{account}/{name}"), percent);
            if settings.recovery
                && previous.is_some_and(|old| {
                    thresholds.iter().any(|threshold| {
                        old < f64::from(*threshold) && percent >= f64::from(*threshold)
                    })
                })
                && self.sent.insert(format!("{cycle}/recovery"))
            {
                messages.push(format!("{name} 额度已恢复至 {percent:.0}%。"));
            }
            if settings.reset_soon
                && reset.is_some_and(|time| {
                    let seconds = (time.with_timezone(&Utc) - now).num_seconds();
                    seconds > 0 && seconds <= 300
                })
                && self.sent.insert(format!("{cycle}/reset-soon"))
            {
                messages.push(format!("{name} 额度将在 5 分钟内重置。"));
            }
        }
        messages
    }
    pub fn local_hour() -> u8 {
        chrono::Local::now().hour() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quota::{QuotaErrorCode, WindowQuota};
    fn snapshot() -> QuotaSnapshot {
        let mut result = QuotaSnapshot::error(QuotaErrorCode::Internal, "unused", false);
        result.error = None;
        result.account = Some("test".into());
        result.five_hour = Some(WindowQuota {
            label: "5h".into(),
            used_percent: Some(95.0),
            reset_at: Some("2026-10-17T00:00:00Z".into()),
        });
        result
    }
    #[test]
    fn threshold_is_once_per_cycle_and_survives_restart() {
        let settings = ReminderSettings {
            enabled: true,
            quiet_start: 0,
            quiet_end: 0,
            ..ReminderSettings::default()
        };
        let now = DateTime::parse_from_rfc3339("2026-10-16T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut state = ReminderState::default();
        assert_eq!(state.evaluate(&snapshot(), &settings, now, 12).len(), 1);
        let mut restored: ReminderState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        assert!(restored
            .evaluate(&snapshot(), &settings, now, 12)
            .is_empty());
        let mut next = snapshot();
        next.five_hour.as_mut().unwrap().reset_at = Some("2026-10-18T00:00:00Z".into());
        assert_eq!(restored.evaluate(&next, &settings, now, 12).len(), 1);
    }
    #[test]
    fn quiet_hours_wrap_midnight_and_do_not_consume_notifications() {
        let settings = ReminderSettings {
            enabled: true,
            ..ReminderSettings::default()
        };
        let now = Utc::now();
        let mut state = ReminderState::default();
        assert!(settings.quiet(23));
        assert!(settings.quiet(6));
        assert!(!settings.quiet(12));
        assert!(state.evaluate(&snapshot(), &settings, now, 23).is_empty());
        assert!(state.sent.is_empty());
    }
}
