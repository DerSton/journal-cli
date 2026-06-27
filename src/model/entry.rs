//! Data model representing a journal entry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single encrypted journal entry.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalEntry {
    /// A unique identifier for the entry, usually a UUID.
    pub id: String,
    /// The creation date and time of the entry in UTC.
    pub timestamp: DateTime<Utc>,
    /// The plain text content of the journal entry.
    pub content: String,
    /// Optional date this entry applies to (for back-dating).
    #[serde(default)]
    pub date_for: Option<chrono::NaiveDate>,
}

impl JournalEntry {
    /// Returns the timestamp used for sorting. If `date_for` is set,
    /// it shifts the date part while keeping the original creation time part.
    pub fn sort_timestamp(&self) -> DateTime<Utc> {
        if let Some(date) = self.date_for {
            use chrono::TimeZone;
            let time_part = self.timestamp.time();
            if let Some(dt) = Utc.from_local_datetime(&date.and_time(time_part)).single() {
                dt
            } else {
                date.and_hms_opt(0, 0, 0)
                    .and_then(|naive| Utc.from_local_datetime(&naive).single())
                    .unwrap_or(self.timestamp)
            }
        } else {
            self.timestamp
        }
    }
}
