use crate::repository::metric_repository::{
    MetricStoreRepository, MetricStoreRequest, MetricsCacheRepository, MetricsWatchRepository,
};
use crate::services::metrics::{MetricsInMemoryCacheService, MetricsWatchService};
use crate::services::studio::Studio;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

pub struct MetricUsecase {
    store_repository: Arc<TokioMutex<dyn MetricStoreRepository + Send + Sync + 'static>>,
    watch_repository: Arc<Mutex<dyn MetricsWatchRepository + Send + Sync + 'static>>,
    cache_repository: Arc<Mutex<MetricsInMemoryCacheService>>,
}

impl MetricUsecase {
    pub fn new() -> Self {
        MetricUsecase {
            store_repository: Arc::new(TokioMutex::new(Studio::new())),
            watch_repository: Arc::new(Mutex::new(MetricsWatchService::new())),
            cache_repository: Arc::new(Mutex::new(MetricsInMemoryCacheService::new())),
        }
    }

    pub async fn start_collect_metric(&mut self) {
        let watch_repository_clone = Arc::clone(&self.watch_repository);
        let cache_repository_clone = Arc::clone(&self.cache_repository);

        let watch_task = tokio::spawn(async move {
            loop {
                let metrics = watch_repository_clone.lock().unwrap().watch_metrics();
                for metric in metrics {
                    cache_repository_clone.lock().unwrap().push(vec![metric]);
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        let store_repository_clone = Arc::clone(&self.store_repository);
        let cache_repository_clone2 = Arc::clone(&self.cache_repository);

        let send_task = tokio::spawn(async move {
            loop {
                let metrics = cache_repository_clone2.lock().unwrap().get();
                for metric in metrics {
                    let request = MetricStoreRequest {
                        device_did: super::get_my_did(),
                        timestamp: metric.timestamp,
                        metric_name: metric.metric_type.to_string(),
                        metric_value: metric.value,
                    };

                    store_repository_clone
                        .lock()
                        .await
                        .save(request)
                        .await
                        .unwrap();
                    cache_repository_clone2.lock().unwrap().clear();
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            }
        });

        tokio::try_join!(watch_task, send_task).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::repository::metric_repository::{
        Metric, MetricStoreRepository, MetricType, MetricsWatchRepository,
    };

    pub struct MockMetricStoreRepository {}

    #[async_trait::async_trait]
    impl MetricStoreRepository for MockMetricStoreRepository {
        async fn save(&self, _: MetricStoreRequest) -> anyhow::Result<()> {
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
                    timestamp: Utc::now(),
                },
                Metric {
                    metric_type: MetricType::MemoryUsage,
                    value: 0.0,
                    timestamp: Utc::now(),
                },
            ]
        }
    }

    #[tokio::test]
    async fn test_start_collect_metric() {
        let mut usecase = MetricUsecase {
            store_repository: Arc::new(TokioMutex::new(MockMetricStoreRepository {})),
            watch_repository: Arc::new(Mutex::new(MockMetricWatchRepository {})),
            cache_repository: Arc::new(Mutex::new(MetricsInMemoryCacheService::new())),
        };
        usecase.start_collect_metric().await;
    }
}
