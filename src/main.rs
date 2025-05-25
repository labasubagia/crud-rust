use std::sync::{Arc, Mutex};

use axum::{Extension, Json, extract::State, http::StatusCode, routing::get};
use serde_json::{self, json};
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use crud_rust::{
    config::Config,
    middleware::{CorrelationId, request_middleware},
    model::http::Response,
    state::AppState,
};

#[tokio::main]
async fn main() {
    let app_state: Arc<AppState> = Arc::new(AppState {
        config: Config::new(),
        items: Mutex::new(Vec::new()),
    });

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        tracing::error!("Failed to set global tracing subscriber: {}", e);
        return;
    }
    let app = setup_app(app_state.clone());

    let addr = app_state.config.get_addr();
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

fn setup_app(app_state: Arc<AppState>) -> axum::Router {
    use crud_rust::handler::item::router_setup_items;

    axum::Router::new()
        .route("/", get(handler_index))
        .route("/api/healthcheck", get(handler_healthcheck))
        .nest("/api/items", router_setup_items())
        .layer(axum::middleware::from_fn(request_middleware))
        .with_state(app_state)
}

async fn handler_index(
    State(app_state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::OK;
    info!(
        app_name = %app_state.config.app_name,
        status_code = status_code.as_u16(),
        correlation_id = %correlation_id,
        "Request handled"
    );
    (
        status_code,
        Json(json!(Response::<serde_json::Value> {
            correlation_id,
            message: format!("Welcome to {}!", app_state.config.app_name),
            error: "".into(),
            data: None,
        })),
    )
}

async fn handler_healthcheck(
    State(app_state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::OK;
    info!(
        app_name = %app_state.config.app_name,
        status_code = status_code.as_u16(),
        correlation_id = %correlation_id,
        "Request handled"
    );
    (
        status_code,
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
        let app_state = Arc::new(AppState::default());
        let app = setup_app(app_state.clone());

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
        let app_state = Arc::new(AppState::default());
        let app = setup_app(app_state.clone());

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
