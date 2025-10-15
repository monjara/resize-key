use serde::{Deserialize, Serialize};

const DEFAULT_JSONC: &[u8] = include_bytes!("data/default.jsonc");

pub(crate) enum Operation {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    ResizeLeftToLeft,
    ResizeLeftToRight,
    ResizeTopToTop,
    ResizeTopToBottom,
    ResizeRightToLeft,
    ResizeRightToRight,
    ResizeBottomToTop,
    ResizeBottomToBottom,
}

impl From<&str> for Operation {
    fn from(s: &str) -> Self {
        match s {
            "move_left" => Operation::MoveLeft,
            "move_right" => Operation::MoveRight,
            "move_up" => Operation::MoveUp,
            "move_down" => Operation::MoveDown,
            "resize_left_to_left" => Operation::ResizeLeftToLeft,
            "resize_left_to_right" => Operation::ResizeLeftToRight,
            "resize_top_to_top" => Operation::ResizeTopToTop,
            "resize_top_to_bottom" => Operation::ResizeTopToBottom,
            "resize_right_to_left" => Operation::ResizeRightToLeft,
            "resize_right_to_right" => Operation::ResizeRightToRight,
            "resize_bottom_to_top" => Operation::ResizeBottomToTop,
            "resize_bottom_to_bottom" => Operation::ResizeBottomToBottom,
            _ => panic!("Unknown operation: {s}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Binding {
    pub(crate) operation: String,
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
