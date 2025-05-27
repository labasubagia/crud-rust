use std::sync::Arc;

use axum::{Extension, Json, extract::State, http::StatusCode, routing::get};
use serde_json::{self, json};
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use crud_rust::{
    config::Config,
    handler::{item::router_setup_items, user::router_setup_users},
    middleware::{CorrelationId, request_middleware},
    model::http::Response,
    repository::PostgresRepository,
    service::Service,
    state::AppState,
};
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        tracing::error!("Failed to set global tracing subscriber: {}", e);
        return;
    }

    let config = Arc::new(Config::new());

    // Use PostgresItemRepository with 'static lifetime by leaking the pool reference
    let pool = match PgPool::connect(&config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            return;
        }
    };

    let repo = Arc::new(PostgresRepository::new(pool.clone()));
    let service = Arc::new(Service::new(config.clone(), repo.clone()));

    let app_state = Arc::new(AppState {
        db_pool: pool.clone(),
        config: config.clone(),
        service: service.clone(),
    });
    let app = setup_app(app_state.clone());

    let addr = &app_state.config.get_addr();
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    info!(
        app_name = %app_state.config.app_name,
        addr = %addr.to_string(),
        "Starting server..."
    );

    if let Err(e) = axum::serve(listener, app.into_make_service()).await {
        tracing::error!("Server error: {}", e);
        return;
    }
}

fn setup_app(state: Arc<AppState>) -> axum::Router {
    axum::Router::new()
        .route("/", get(handler_index))
        .route("/api/healthcheck", get(handler_healthcheck))
        .nest("/api/items", router_setup_items())
        .nest("/api/users", router_setup_users())
        .layer(axum::middleware::from_fn(request_middleware))
        .with_state(state)
}

async fn handler_index(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!(Response::<serde_json::Value> {
            correlation_id,
            message: format!("Welcome to {}!", &state.config.app_name),
            error: "".into(),
            data: None,
        })),
    )
}

async fn handler_healthcheck(
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!(Response::<serde_json::Value> {
            correlation_id,
            message: "ok".into(),
            error: "".into(),
            data: None,
        })),
    )
}
