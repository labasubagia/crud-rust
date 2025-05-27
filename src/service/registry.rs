use std::sync::Arc;

use crate::repository::Repository;

use super::{item::ItemService, user::UserService};
use crate::config::Config;

pub struct Service {
    pub config: Arc<Config>,
    pub item: ItemService,
    pub user: UserService,
}

impl Service {
    pub fn new(config: Arc<Config>, repo: Arc<dyn Repository>) -> Self {
        Self {
            config: config.clone(),
            item: ItemService::new(config.clone(), repo.clone()),
            user: UserService::new(config.clone(), repo.clone()),
        }
    }
}
