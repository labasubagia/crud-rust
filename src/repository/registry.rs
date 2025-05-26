use std::sync::Arc;

use super::item::ItemRepository;

pub struct Repository {
    pub item: Arc<dyn ItemRepository>,
}
