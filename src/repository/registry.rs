use std::sync::Arc;

use super::{item::ItemRepository, user::UserRepository};

pub struct Repository {
    pub item: Arc<dyn ItemRepository>,
    pub user: Arc<dyn UserRepository>,
}

impl Repository {
    pub fn new(item: Arc<dyn ItemRepository>, user: Arc<dyn UserRepository>) -> Self {
        Self { item, user }
    }
}
