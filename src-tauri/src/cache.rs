use crate::quota::{QuotaError, QuotaSnapshot};
use crate::settings::atomic_write;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Default, Serialize, Deserialize)]
pub struct SnapshotCache {
    success: Option<QuotaSnapshot>,
}

impl SnapshotCache {
    pub fn load(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        atomic_write(path, &serde_json::to_vec(self).map_err(|e| e.to_string())?)
    }

    pub fn accept(&mut self, mut result: QuotaSnapshot) -> QuotaSnapshot {
        if result.error.is_none() {
            result.stale = false;
            self.success = Some(result.clone());
            self.view(Utc::now(), 300).unwrap_or(result)
        } else {
            self.fallback(result.error.take().unwrap(), result)
        }
    }

    fn fallback(&self, error: QuotaError, mut empty: QuotaSnapshot) -> QuotaSnapshot {
        if let Some(previous) = &self.success {
            if empty.account.is_some() && empty.account != previous.account {
                empty.error = Some(error);
                return empty;
            }
            let mut snapshot = previous.clone();
            snapshot.stale = true;
            snapshot.error = Some(error);
            snapshot
        } else {
            empty.error = Some(error);
            empty
        }
    }

    pub fn view(&self, now: DateTime<Utc>, max_age_secs: i64) -> Option<QuotaSnapshot> {
        self.success.clone().map(|mut snapshot| {
            snapshot.stale = snapshot
                .updated_at
                .as_deref()
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .is_none_or(|updated| {
                    (now - updated.with_timezone(&Utc)).num_seconds() > max_age_secs
                        || updated > now
                });
            snapshot
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quota::{QuotaErrorCode, WindowQuota};
    #[test]
    fn successful_cli_call_does_not_make_old_source_data_fresh() {
        let mut snapshot = success();
        snapshot.updated_at = Some((Utc::now() - chrono::Duration::hours(1)).to_rfc3339());
        let mut cache = SnapshotCache::default();
        assert!(cache.accept(snapshot).stale);
    }

    fn success() -> QuotaSnapshot {
        let mut result = QuotaSnapshot::error(QuotaErrorCode::Internal, "unused", false);
        result.error = None;
        result.updated_at = Some("2026-10-16T00:00:00Z".into());
        result.account = Some("test-account".into());
        result.five_hour = Some(WindowQuota {
            label: "5h".into(),
            used_percent: Some(31.0),
            reset_at: None,
        });
        result
    }

    #[test]
    fn failure_keeps_last_success_and_its_original_timestamp() {
        let mut cache = SnapshotCache::default();
        cache.accept(success());
        let fallback = cache.accept(QuotaSnapshot::error(
            QuotaErrorCode::Internal,
            "offline",
            true,
        ));
        assert!(fallback.stale);
        assert_eq!(fallback.updated_at.as_deref(), Some("2026-10-16T00:00:00Z"));
        assert_eq!(fallback.five_hour.unwrap().used_percent, Some(31.0));
    }

    #[test]
    fn expiry_and_restart_round_trip() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("snapshot.json");
        let mut cache = SnapshotCache::default();
        cache.accept(success());
        cache.save(&path).unwrap();
        let restored = SnapshotCache::load(&path);
        let now = DateTime::parse_from_rfc3339("2026-10-16T00:05:01Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(restored.view(now, 300).unwrap().stale);
    }

    #[test]
    fn failure_without_success_never_creates_quota() {
        let mut cache = SnapshotCache::default();
        let result = cache.accept(QuotaSnapshot::error(
            QuotaErrorCode::Internal,
            "offline",
            true,
        ));
        assert!(result.five_hour.is_none());
        assert!(!result.stale);
    }
}
