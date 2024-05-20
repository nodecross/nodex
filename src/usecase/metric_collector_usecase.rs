use std::sync::Arc;

use crate::repository::metric_repository::{Metric, MetricCollectRepository};
use tokio::sync::{mpsc, Notify};

pub struct MetricCollectorUsecase {
    repository: Box<dyn MetricCollectRepository + Send + 'static>,
    sender: mpsc::Sender<Metric>,
    interval: u64,
}

impl MetricCollectorUsecase {
    pub fn new(
        repository: Box<dyn MetricCollectRepository + Send + 'static>,
        sender: mpsc::Sender<Metric>,
        interval: u64,
    ) -> Self {
        MetricCollectorUsecase {
            repository,
            sender,
            interval,
        }
    }

    pub async fn start_collect(&mut self, shutdown_notify: Arc<Notify>) {
        loop {
            tokio::select! {
                _ = shutdown_notify.notified() => {
                    break;
                },
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(self.interval)) => {
                    let metrics = self.repository.collect();
                    println!("Collected metrics: {:?}", metrics);
                    for metric in metrics {
                        let _ = self.sender.send(metric).await;
                    }
                }
            }
        }
    }
}
