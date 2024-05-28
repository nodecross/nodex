use crate::app_config;
use crate::config::SingletonAppConfig;
use crate::repository::metric_repository::{
    MetricStoreRepository, MetricStoreRequest, MetricsCacheRepository, MetricsWatchRepository,
};
use crate::services::metrics::{MetricsInMemoryCacheService, MetricsWatchService};
use crate::services::studio::Studio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

pub struct MetricUsecase {
    store_repository: Arc<TokioMutex<dyn MetricStoreRepository + Send + Sync + 'static>>,
    watch_repository: Arc<Mutex<dyn MetricsWatchRepository + Send + Sync + 'static>>,
    cache_repository: Arc<Mutex<MetricsInMemoryCacheService>>,
    config: Box<SingletonAppConfig>,
    should_stop: Arc<AtomicBool>,
}

impl MetricUsecase {
    pub fn new(should_stop: Arc<AtomicBool>) -> Self {
        MetricUsecase {
            store_repository: Arc::new(TokioMutex::new(Studio::new())),
            watch_repository: Arc::new(Mutex::new(MetricsWatchService::new())),
            cache_repository: Arc::new(Mutex::new(MetricsInMemoryCacheService::new())),
            config: app_config(),
            should_stop,
        }
    }

    pub async fn start_send_metric(&mut self) {
        let watch_repository_clone = Arc::clone(&self.watch_repository);
        let cache_repository_clone = Arc::clone(&self.cache_repository);
        let interval: u64 = self.config.lock().get_metric_collect_interval();

        let should_stop_clone = self.should_stop.clone();
        let watch_task = tokio::spawn(async move {
            while !should_stop_clone.load(Ordering::Relaxed) {
                let metrics = watch_repository_clone.lock().unwrap().watch_metrics();
                for metric in metrics {
                    cache_repository_clone.lock().unwrap().push(vec![metric]);
                }
                log::info!("collected metrics");

                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            }
        });

        let store_repository_clone = Arc::clone(&self.store_repository);
        let cache_repository_clone = Arc::clone(&self.cache_repository);
        let interval: u64 = self.config.lock().get_metric_send_interval();

        let should_stop_clone = self.should_stop.clone();
        let send_task = tokio::spawn(async move {
            while !should_stop_clone.load(Ordering::Relaxed) {
                let metrics = cache_repository_clone.lock().unwrap().get();
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
                    cache_repository_clone.lock().unwrap().clear();
                }
                log::info!("sended metrics");

                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
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
            config: app_config(),
        };
        usecase.start_send_metric().await;
    }
}
