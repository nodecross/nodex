use chrono::{DateTime, Utc};

pub fn milliseconds_to_time(milliseconds: u64) -> Option<DateTime<Utc>> {
    let milliseconds = milliseconds as i64;
    match milliseconds.to_string().len() {
        13 => {
            let secs = milliseconds / 1000;
            let nsecs = (milliseconds % 1000) * 1_000_000;
            DateTime::from_timestamp(secs, nsecs as u32)
        }
        _ => None,
    }
}
