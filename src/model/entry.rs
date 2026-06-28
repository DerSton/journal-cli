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
    /// it treats the entry as written at 23:59:59 local time on that date.
    pub fn sort_timestamp(&self) -> DateTime<Utc> {
        if let Some(date) = self.date_for {
            use chrono::TimeZone;
            let local_time = chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap();
            let local_dt = date.and_time(local_time);
            if let Some(dt) = chrono::Local.from_local_datetime(&local_dt).single() {
                dt.with_timezone(&Utc)
            } else {
                date.and_hms_opt(23, 59, 59)
                    .and_then(|naive| Utc.from_local_datetime(&naive).single())
                    .unwrap_or(self.timestamp)
            }
        } else {
            self.timestamp
        }
    }
}
