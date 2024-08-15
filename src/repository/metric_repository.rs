use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{
    collections::VecDeque,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug, Serialize, Clone)]
pub struct Metric {
    pub metric_type: MetricType,
    pub value: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct MetricsWithTimestamp {
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<Metric>,
}

pub trait MetricsWatchRepository {
    fn watch_metrics(&mut self) -> Vec<Metric>;
}

#[trait_variant::make(Send)]
pub trait MetricsCacheRepository {
    async fn push(&mut self, timestamp: DateTime<Utc>, metrics: Vec<Metric>);
    async fn clear(&mut self);
    async fn get(&mut self) -> VecDeque<MetricsWithTimestamp>;
}

#[trait_variant::make(Send)]
pub trait MetricStoreRepository {
    async fn save(&self, request: VecDeque<MetricsWithTimestamp>) -> anyhow::Result<()>;
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
