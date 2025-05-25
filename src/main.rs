use std::sync::Arc;

use axum::{Extension, Json, extract::State, http::StatusCode, routing::get};
use serde_json::{self, json};
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use crud_rust::{
    config::Config,
    middleware::{CorrelationId, request_middleware},
    model::http::Response,
    repository::item::{InMemoryItemRepository, ItemRepository},
    service::item::ItemService,
};

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
    let repo = Arc::new(InMemoryItemRepository::new());
    let service = Arc::new(ItemService::new(config.clone(), repo.clone()));
    let app = setup_app(service.clone());

    let addr = service.config.get_addr();
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    info!(
        app_name = %service.config.app_name,
        addr = %addr.to_string(),
        "Starting server..."
    );

    if let Err(e) = axum::serve(listener, app.into_make_service()).await {
        tracing::error!("Server error: {}", e);
        return;
    }
}

fn setup_app<R>(service: Arc<ItemService<R>>) -> axum::Router
where
    R: ItemRepository + 'static,
{
    use crud_rust::handler::item::router_setup_items;
    axum::Router::new()
        .route("/", get(handler_index))
        .route("/api/healthcheck", get(handler_healthcheck))
        .nest("/api/items", router_setup_items())
        .layer(axum::middleware::from_fn(request_middleware))
        .with_state(service)
}

async fn handler_index<R>(
    State(service): State<Arc<ItemService<R>>>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>)
where
    R: ItemRepository,
{
    (
        StatusCode::OK,
        Json(json!(Response::<serde_json::Value> {
            correlation_id,
            message: format!("Welcome to {}!", service.config.app_name),
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_index_handler() {
        let service = Arc::new(ItemService::new(
            Arc::new(Config::new()),
            Arc::new(InMemoryItemRepository::new()),
        ));
        let app = setup_app(service.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["message"], "Welcome to my_app!");
        assert_eq!(json["error"], "");
        assert_eq!(json["data"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_healthcheck_handler() {
        let service = Arc::new(ItemService::new(
            Arc::new(Config::new()),
            Arc::new(InMemoryItemRepository::new()),
        ));
        let app = setup_app(service.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/healthcheck")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["message"], "ok");
        assert_eq!(json["error"], "");
        assert_eq!(json["data"], serde_json::Value::Null);
    }
}
