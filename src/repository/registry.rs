use std::sync::Arc;

use super::item::ItemRepository;

pub struct Repository {
    pub item: Arc<dyn ItemRepository>,
}

impl Repository {
    pub fn new(item: Arc<dyn ItemRepository>) -> Self {
        Self { item }
    }
}
