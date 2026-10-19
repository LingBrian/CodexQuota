use crate::{
    cache::SnapshotCache,
    quota::{QuotaErrorCode, QuotaSnapshot},
};
use chrono::Utc;
use std::{
    future::Future,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{watch, Mutex as AsyncMutex};

#[derive(Clone)]
pub struct RefreshCoordinator {
    inner: Arc<Inner>,
}
struct Inner {
    gate: AsyncMutex<()>,
    cache: Mutex<SnapshotCache>,
    last_attempt: Mutex<Option<Instant>>,
    last_result: Mutex<Option<QuotaSnapshot>>,
    cancel: watch::Sender<u64>,
    path: PathBuf,
}
impl RefreshCoordinator {
    pub async fn exclusive(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.inner.gate.lock().await
    }
    pub fn invalidate(&self) -> Result<(), String> {
        let mut cache = self.inner.cache.lock().map_err(|_| "缓存状态不可用")?;
        *cache = SnapshotCache::default();
        *self
            .inner
            .last_result
            .lock()
            .map_err(|_| "刷新状态不可用")? = None;
        *self
            .inner
            .last_attempt
            .lock()
            .map_err(|_| "刷新状态不可用")? = None;
        cache.save(&self.inner.path)
    }
    pub fn new(path: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                gate: AsyncMutex::new(()),
                cache: Mutex::new(SnapshotCache::load(&path)),
                last_attempt: Mutex::new(None),
                last_result: Mutex::new(None),
                cancel: watch::channel(0).0,
                path,
            }),
        }
    }
    pub fn cancel(&self) {
        self.inner
            .cancel
            .send_modify(|value| *value = value.wrapping_add(1));
    }
    pub fn cached(&self) -> Option<QuotaSnapshot> {
        self.inner.cache.lock().ok()?.view(Utc::now(), 300)
    }
    fn failure(&self, code: QuotaErrorCode, text: &str) -> QuotaSnapshot {
        let error = QuotaSnapshot::error(code, text, true);
        self.inner
            .cache
            .lock()
            .map(|mut cache| cache.accept(error.clone()))
            .unwrap_or(error)
    }
    pub async fn run<F, Fut>(
        &self,
        min_interval: Duration,
        timeout: Duration,
        fetch: F,
    ) -> QuotaSnapshot
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = QuotaSnapshot>,
    {
        let Ok(_guard) = self.inner.gate.try_lock() else {
            // A tray click and timer can overlap. Share the completed result;
            // do not broadcast a fake query failure while the provider is working.
            let _wait = self.inner.gate.lock().await;
            return self
                .inner
                .last_result
                .lock()
                .unwrap()
                .clone()
                .or_else(|| self.cached())
                .unwrap_or_else(|| {
                    QuotaSnapshot::error(
                        QuotaErrorCode::RefreshInProgress,
                        "账号操作正在完成，请稍后刷新",
                        true,
                    )
                });
        };
        if self
            .inner
            .last_attempt
            .lock()
            .unwrap()
            .is_some_and(|last| last.elapsed() < min_interval)
        {
            return self
                .inner
                .last_result
                .lock()
                .unwrap()
                .clone()
                .or_else(|| self.cached())
                .unwrap_or_else(|| {
                    self.failure(QuotaErrorCode::RateLimited, "刷新过于频繁，请稍后再试")
                });
        }
        *self.inner.last_attempt.lock().unwrap() = Some(Instant::now());
        let mut cancelled = self.inner.cancel.subscribe();
        let result = tokio::select! {
            _ = cancelled.changed() => QuotaSnapshot::error(QuotaErrorCode::Cancelled, "查询已取消", true),
            result = tokio::time::timeout(timeout, fetch()) => result.unwrap_or_else(|_| QuotaSnapshot::error(QuotaErrorCode::Timeout, "额度查询超时", true)),
        };
        let mut cache = self.inner.cache.lock().unwrap();
        let mut snapshot = cache.accept(result);
        if snapshot.error.is_none() && cache.save(&self.inner.path).is_err() {
            snapshot.error = Some(crate::quota::QuotaError {
                code: QuotaErrorCode::CacheWrite,
                message: "额度查询成功，但本地缓存保存失败".into(),
                retryable: true,
            });
        }
        *self.inner.last_result.lock().unwrap() = Some(snapshot.clone());
        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn timeout_is_bounded_and_gate_is_released() {
        let directory = tempfile::tempdir().unwrap();
        let coordinator = RefreshCoordinator::new(directory.path().join("cache.json"));
        let result = coordinator
            .run(Duration::ZERO, Duration::from_millis(5), || async {
                tokio::time::sleep(Duration::from_secs(60)).await;
                unreachable!()
            })
            .await;
        assert!(matches!(
            result.error.unwrap().code,
            QuotaErrorCode::Timeout
        ));
        let result = coordinator
            .run(Duration::ZERO, Duration::from_secs(1), || async {
                QuotaSnapshot::error(QuotaErrorCode::NoAccounts, "empty", false)
            })
            .await;
        assert!(matches!(
            result.error.unwrap().code,
            QuotaErrorCode::NoAccounts
        ));
    }
    #[tokio::test]
    async fn concurrent_refresh_is_deduplicated_and_cancellable() {
        let directory = tempfile::tempdir().unwrap();
        let coordinator = RefreshCoordinator::new(directory.path().join("cache.json"));
        let worker = coordinator.clone();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let job = tokio::spawn(async move {
            worker
                .run(Duration::ZERO, Duration::from_secs(60), || async {
                    started_tx.send(()).unwrap();
                    std::future::pending().await
                })
                .await
        });
        started_rx.await.unwrap();
        let duplicate_worker = coordinator.clone();
        let duplicate = tokio::spawn(async move {
            duplicate_worker
                .run(Duration::ZERO, Duration::from_secs(1), || async {
                    panic!("duplicate ran")
                })
                .await
        });
        tokio::task::yield_now().await;
        coordinator.cancel();
        let cancelled = job.await.unwrap();
        assert!(matches!(
            cancelled.error.unwrap().code,
            QuotaErrorCode::Cancelled
        ));
        assert!(matches!(
            duplicate.await.unwrap().error.unwrap().code,
            QuotaErrorCode::Cancelled
        ));
    }
}
