use std::sync::Arc;

use crate::repository::Repository;

use crate::config::Config;

pub struct Service {
    pub config: Arc<Config>,
    pub repo: Arc<dyn Repository>,
}

impl Service {
    pub fn new(config: Arc<Config>, repo: Arc<dyn Repository>) -> Self {
        Self { config, repo }
    }
}
