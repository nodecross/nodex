use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Serialize, Clone)]
pub struct Metric {
    pub metric_type: MetricType,
    pub value: f32,
    pub timestamp: DateTime<Utc>,
}

pub trait MetricsWatchRepository {
    fn watch_metrics(&mut self) -> Vec<Metric>;
}

pub trait MetricsCacheRepository {
    fn new() -> Self;
    fn push(&mut self, metrics: Vec<Metric>);
    fn clear(&mut self);
    fn get(&mut self) -> Vec<Metric>;
}

#[derive(Debug, Serialize, Clone)]
pub struct MetricStoreRequest {
    pub device_did: String,
    pub timestamp: DateTime<Utc>,
    pub metric_name: String,
    pub metric_value: f32,
}

#[async_trait::async_trait]
pub trait MetricStoreRepository {
    async fn save(&self, request: MetricStoreRequest) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum MetricType {
    CpuUsage,
    MemoryUsage,
    NetworkReceivedBytes,
    NetworkTransmittedBytes,
    NetworkReceivedPackets,
    NetworkTransmittedPackets,
    DiskReadBytes,
    DiskWrittenBytes,
}

impl Display for MetricType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            MetricType::CpuUsage => write!(f, "cpu_usage"),
            MetricType::MemoryUsage => write!(f, "memory_usage"),
            MetricType::NetworkReceivedBytes => write!(f, "network_received_bytes"),
            MetricType::NetworkTransmittedBytes => write!(f, "network_transmitted_bytes"),
            MetricType::NetworkReceivedPackets => write!(f, "network_received_packets"),
            MetricType::NetworkTransmittedPackets => write!(f, "network_transmitted_packets"),
            MetricType::DiskReadBytes => write!(f, "disk_read_bytes"),
            MetricType::DiskWrittenBytes => write!(f, "disk_written_bytes"),
        }
    }
}
