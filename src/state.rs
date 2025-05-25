use std::sync::Mutex;

use crate::{config::Config, model::item::Item};

pub struct AppState {
    pub config: Config,
    pub items: Mutex<Vec<Item>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            config: Config::default(),
            items: Mutex::new(Vec::new()),
        }
    }
}
