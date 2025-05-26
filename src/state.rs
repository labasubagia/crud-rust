use std::sync::Arc;

use sqlx::PgPool;

use crate::{config::Config, service::Service};

pub struct AppState {
    pub db_pool: PgPool,
    pub config: Arc<Config>,
    pub service: Arc<Service>,
}
