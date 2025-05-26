use std::sync::Arc;

use crate::{config::Config, service::Service};

pub struct AppState {
    pub config: Arc<Config>,
    pub service: Arc<Service>,
}
