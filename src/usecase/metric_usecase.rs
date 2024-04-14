use crate::repository::metric_repository::{
    MetricStoreRepository, MetricStoreRequest, MetricsWatchRepository,
};
use crate::services::hub::Hub;
use crate::services::metrics::MetricsWatchService;
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct MetricUsecase {
    store_repository: Box<dyn MetricStoreRepository + Send + Sync + 'static>,
    repository: Box<dyn MetricsWatchRepository + Send + Sync + 'static>,
    should_stop: Arc<AtomicBool>,
}

impl MetricUsecase {
    pub fn new(should_stop: Arc<AtomicBool>) -> Self {
        MetricUsecase {
            store_repository: Box::new(Hub::new()),
            repository: Box::new(MetricsWatchService::new()),
            should_stop,
        }
    }

    async fn send_request(&self, metric_name: &str, metric_value: f32, timestamp: DateTime<Utc>) {
        let request = MetricStoreRequest {
            device_did: super::get_my_did(),
            timestamp,
            metric_name: metric_name.to_string(),
            metric_value,
        };

        match self.store_repository.save(request).await {
            Ok(_) => log::info!("save {}", metric_name),
            Err(e) => log::error!("{:?}", e),
        }
    }

    pub async fn start_collect_metric(&mut self) {
        while !self.should_stop.load(Ordering::Relaxed) {
            let metrics = self.repository.watch_metrics();

            for metric in metrics {
                self.send_request(
                    metric.metric_type.to_string().as_str(),
                    metric.value,
                    metric.timestamp,
                )
                .await;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
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
            store_repository: Box::new(MockMetricStoreRepository {}),
            repository: Box::new(MockMetricWatchRepository {}),
            should_stop: Arc::new(AtomicBool::new(false)),
        };

        usecase.should_stop.store(true, Ordering::Relaxed);
        usecase.start_collect_metric().await;
    }
}
