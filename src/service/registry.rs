use std::sync::Arc;

use crate::repository::Repository;

use super::item::ItemService;
use crate::config::Config;

pub struct Service {
    pub config: Arc<Config>,
    pub item: ItemService,
}

impl Service {
    pub fn new(config: Arc<Config>, repo: Arc<Repository>) -> Self {
        Self {
            config: config.clone(),
            item: ItemService::new(config.clone(), repo.clone()),
        }
    }
}
