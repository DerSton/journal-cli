//! Data model representing the user settings.

use serde::{Deserialize, Serialize};

fn default_autolock_timeout() -> u32 {
    5
}

fn default_lock_on_suspend() -> bool {
    true
}

fn default_ollama_enabled() -> bool {
    false
}

fn default_ollama_model() -> String {
    "llama3".to_string()
}

fn default_ollama_days() -> u32 {
    7
}

/// Persistent user configuration settings.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    /// Inactivity auto-lock timeout in minutes. Set to 0 to disable.
    #[serde(default = "default_autolock_timeout")]
    pub autolock_timeout_mins: u32,
    /// Automatically lock the journal when the system workstation suspends (Windows only).
    #[serde(default = "default_lock_on_suspend")]
    pub lock_on_suspend: bool,
    /// Whether Ollama-based summary is enabled on the Dashboard.
    #[serde(default = "default_ollama_enabled")]
    pub ollama_enabled: bool,
    /// The Ollama model name used for summary.
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
    /// Number of days of journal entries to use for the AI summary.
    #[serde(default = "default_ollama_days")]
    pub ollama_days: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autolock_timeout_mins: default_autolock_timeout(),
            lock_on_suspend: default_lock_on_suspend(),
            ollama_enabled: default_ollama_enabled(),
            ollama_model: default_ollama_model(),
            ollama_days: default_ollama_days(),
        }
    }
}
