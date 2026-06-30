//! Data model representing a journal entry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A file attachment embedded in a journal entry.
///
/// The file content is stored as a Base64-encoded string to remain
/// compatible with JSON serialization.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attachment {
    /// Original file name (e.g. "photo.jpg").
    pub filename: String,
    /// MIME type of the file (e.g. "image/jpeg").
    pub mime_type: String,
    /// File size in bytes (before Base64 encoding).
    pub size_bytes: u64,
    /// Base64-encoded file content.
    pub data: String,
}

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
    /// File attachments embedded in this entry.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
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
