use serde::{Deserialize, Serialize};

const DEFAULT_JSONC: &[u8] = include_bytes!("data/default.jsonc");

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Binding {
    pub(crate) action: String,
    pub(crate) key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Preferences {
    pub(crate) move_step: f64,
    pub(crate) resize_step: f64,
    pub(crate) bindings: Vec<Binding>,
}

impl Preferences {
    pub(crate) fn new() -> Self {
        let Ok(app_name) = std::env::var("CARGO_PKG_NAME") else {
            return Self::load_default();
        };

        let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME").map(std::path::PathBuf::from)
        else {
            return Self::load_default();
        };
        let config_path = xdg_config_home.join(app_name).join("settings.json");
        let Ok(config_data) = std::fs::read(&config_path) else {
            return Self::load_default();
        };
        let Ok(pref) = serde_json::from_slice(&config_data) else {
            return Self::load_default();
        };
        pref
    }

    fn load_default() -> Self {
        serde_json::from_slice(DEFAULT_JSONC).expect("Cannot parse default.jsonc")
    }
}
