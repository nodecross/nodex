use crate::config::SingletonAppConfig;
use crate::repository::metric_repository::{
    MetricStoreRepository, MetricsCacheRepository, MetricsWatchRepository,
};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct MetricUsecase<S, W, C>
where
    S: MetricStoreRepository,
    W: MetricsWatchRepository,
    C: MetricsCacheRepository,
{
    store_repository: S,
    watch_repository: W,
    config: Box<SingletonAppConfig>,
    cache_repository: C,
    shutdown_token: CancellationToken,
}

impl<S, W, C> MetricUsecase<S, W, C>
where
    S: MetricStoreRepository,
    W: MetricsWatchRepository,
    C: MetricsCacheRepository,
{
    pub fn new(
        store_repository: S,
        watch_repository: W,
        config: Box<SingletonAppConfig>,
        cache_repository: C,
        shutdown_token: CancellationToken,
    ) -> Self {
        MetricUsecase {
            store_repository,
            watch_repository,
            config,
            cache_repository,
            shutdown_token,
        }
    }

    pub async fn collect_task(&mut self) {
        let interval_time: u64 = self.config.lock().get_metric_collect_interval();
        let mut interval = tokio::time::interval(Duration::from_secs(interval_time));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let metrics = self.watch_repository.watch_metrics();
                    for metric in metrics {
                        self.cache_repository.push(chrono::Utc::now(), vec![metric]).await;
                    }
                    log::info!("collected metrics");
                }
                _ = self.shutdown_token.cancelled() => {
                    break;
                },
            }
        }
    }

    pub async fn send_task(&mut self) {
        let interval_time: u64 = self.config.lock().get_metric_send_interval();
        let mut interval = tokio::time::interval(Duration::from_secs(interval_time));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let metrics_with_timestamp_list = self.cache_repository.get().await;

                    if metrics_with_timestamp_list.is_empty() {
                        continue;
                    }

                    match self.store_repository.save(metrics_with_timestamp_list).await {
                        Ok(_) => {
                            self.cache_repository.clear().await;
                            log::info!("sent metrics");
                        },
                        Err(e) => log::error!("failed to send metric{:?}", e),
                    }
                }
                _ = self.shutdown_token.cancelled() => {
                    break;
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;
    use crate::services::metrics::MetricsInMemoryCacheService;
    use crate::{
        app_config,
        repository::metric_repository::{
            Metric, MetricStoreRepository, MetricType, MetricsWatchRepository, MetricsWithTimestamp,
        },
    };

    pub struct MockMetricStoreRepository {}

    impl MetricStoreRepository for MockMetricStoreRepository {
        async fn save(&self, _: VecDeque<MetricsWithTimestamp>) -> anyhow::Result<()> {
            Ok(())
        }
    }

    pub struct MockMetricWatchRepository {}

    impl MetricsWatchRepository for MockMetricWatchRepository {
        fn watch_metrics(&mut self) -> Vec<Metric> {
            vec![
                Metric {
                    metric_type: MetricType::CpuUsage,
                    value: 0.0,
                },
                Metric {
                    metric_type: MetricType::MemoryUsage,
                    value: 0.0,
                },
            ]
        }
    }

    #[tokio::test]
    async fn test_collect_task() {
        let token = CancellationToken::new();
        let cloned_token = token.clone();
        let mut usecase = MetricUsecase {
            store_repository: MockMetricStoreRepository {},
            watch_repository: MockMetricWatchRepository {},
            config: app_config(),
            cache_repository: MetricsInMemoryCacheService::new(1 << 16),
            shutdown_token: cloned_token,
        };
        token.cancel();
        usecase.collect_task().await;
    }

    #[tokio::test]
    async fn test_send_task() {
        let token = CancellationToken::new();
        let cloned_token = token.clone();
        let mut usecase = MetricUsecase {
            store_repository: MockMetricStoreRepository {},
            watch_repository: MockMetricWatchRepository {},
            config: app_config(),
            cache_repository: MetricsInMemoryCacheService::new(1 << 16),
            shutdown_token: cloned_token,
        };
        token.cancel();
        usecase.send_task().await;
    }
}
