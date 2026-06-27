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
}
