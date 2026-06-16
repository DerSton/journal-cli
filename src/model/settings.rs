use serde::{Deserialize, Serialize};

fn default_autolock_timeout() -> u32 {
    5
}

fn default_lock_on_suspend() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    #[serde(default = "default_autolock_timeout")]
    pub autolock_timeout_mins: u32,
    #[serde(default = "default_lock_on_suspend")]
    pub lock_on_suspend: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autolock_timeout_mins: default_autolock_timeout(),
            lock_on_suspend: default_lock_on_suspend(),
        }
    }
}
