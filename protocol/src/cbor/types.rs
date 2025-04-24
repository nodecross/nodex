use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeValue<T>(#[serde(with = "ts_milliseconds")] pub DateTime<Utc>, pub T);

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomMetric {
    pub key: String,
    pub values: Vec<TimeValue<f32>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub key: String,
    pub details: Vec<TimeValue<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    pub message: TimeValue<String>,
}
