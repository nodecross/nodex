use crate::repository::metric_repository::{
    Metric, MetricFileStoreRepository, MetricInmemoryStoreRepository, MetricSendRepository,
    MetricSendRepositoryImpl, MetricStoreRepository,
};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Notify;

pub struct MetricSenderUsecase {
    // inmemory_store_repository: Box<dyn MetricStoreRepository + Send + 'static>,
    inmemory_store_repository: Arc<Mutex<dyn MetricStoreRepository + Send + 'static>>,
    file_store_repository: Box<dyn MetricStoreRepository + Send + 'static>,
    send_repository: Box<dyn MetricSendRepository + Send + Sync + 'static>,
    // receiver: mpsc::Receiver<Metric>,
    receiver: Arc<Mutex<mpsc::Receiver<Metric>>>,
    interval: u64,
}

impl MetricSenderUsecase {
    pub fn new(
        // inmemory_store_repository: Box<dyn MetricStoreRepository + Send + 'static>,
        inmemory_store_repository: Arc<Mutex<dyn MetricStoreRepository + Send + 'static>>,
        file_store_repository: Box<dyn MetricStoreRepository + Send + 'static>,
        send_repository: Box<dyn MetricSendRepository + Send + Sync + 'static>,
        // receiver: mpsc::Receiver<Metric>,
        receiver: Arc<Mutex<mpsc::Receiver<Metric>>>,
        interval: u64,
    ) -> Self {
        MetricSenderUsecase {
            inmemory_store_repository,
            file_store_repository,
            send_repository,
            receiver,
            interval,
        }
    }

    pub async fn listen(&mut self) {}

    pub async fn start_send(&'static mut self, shutdown_notify: Arc<Notify>) {
        let in_memory_store = Arc::clone(&self.inmemory_store_repository);
        let receiver = Arc::clone(&self.receiver);

        let task1 = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_notify.notified() => {
                        let metrics = self.inmemory_store_repository.lock().unwrap().get_all();
                        self.file_store_repository.set(metrics).unwrap();
                        break;
                    },
                    _ = tokio::time::sleep(Duration::from_secs(self.interval)) => {
                        println!("============================");
                        let metrics = self.inmemory_store_repository.lock().unwrap().get_all();
                        let _ = self.send_repository.send(metrics).await;
                        self.inmemory_store_repository.lock().unwrap().flush().unwrap();
                    },
                }
            }
        });

        // let mut locked_receiver = receiver.lock().unwrap();
        let task2 = tokio::spawn(async move {
            loop {
                let mut locked_receiver = receiver.lock().unwrap();
                tokio::select! {
                    metric = locked_receiver.recv()=> {
                        let mut metrics = vec![];
                        if let Some(metric) = metric {
                            metrics.push(metric);
                        }
                        println!("Received metric: {:?}", metrics);
                        in_memory_store.lock().unwrap().set(metrics).unwrap();
                    }
                }
            }
        });
    }
}
