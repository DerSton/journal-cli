//! Data models representing the core entities of the journal database.

mod contact;
mod entry;
mod journal;
mod settings;

pub use contact::{BLOOD_TYPE_OPTIONS, Contact, MARITAL_STATUS_OPTIONS};
pub use entry::JournalEntry;
pub use journal::{Journal, get_system_locale};
pub use settings::Settings;
