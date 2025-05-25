use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{Extension, Json, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod middleware;
use middleware::{CorrelationId, request_middleware};

mod items;
use items::Item;

pub struct Config {
    pub app_name: String,
    pub host: IpAddr,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_name: "my_app".into(),
            host: Ipv4Addr::new(0, 0, 0, 0).into(),
            port: 3000,
        }
    }
}

impl Config {
    fn new() -> Self {
        let default = Self::default();

        let app_name = env::var("APP_NAME").unwrap_or(default.app_name);
        let host = env::var("HOST")
            .unwrap_or(default.host.to_string())
            .parse::<IpAddr>()
            .unwrap_or(default.host);
        let port = env::var("PORT")
            .unwrap_or(default.port.to_string())
            .parse::<u16>()
            .unwrap_or(default.port);

        Self {
            host,
            port,
            app_name,
        }
    }

    fn get_addr(&self) -> SocketAddr {
        SocketAddr::from((self.host, self.port))
    }
}

pub struct AppState {
    pub config: Config,
    pub items: Mutex<Vec<Item>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            config: Config::default(),
            items: Mutex::new(Vec::new()),
        }
    }
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState {
        config: Config::new(),
        items: Mutex::new(Vec::new()),
    });

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
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
    use items::router_setup as items_router_setup;

    axum::Router::new()
        .route("/", get(handler_index))
        .route("/api/healthcheck", get(handler_healthcheck))
        .nest("/api/items", items_router_setup())
        .layer(axum::middleware::from_fn(request_middleware))
        .with_state(app_state)
}

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub correlation_id: String,
    pub message: String,
    pub error: String,
    pub data: Option<T>,
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

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.app_name, "my_app");
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    }

    #[test]
    fn test_config_with_env() {
        unsafe { env::set_var("APP_NAME", "test_app") };
        unsafe { env::set_var("HOST", "127.0.0.1") };

        let config = Config::new();
        assert_eq!(config.app_name, "test_app");
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn test_config_get_addr() {
        let config = Config::default();
        let addr = config.get_addr();
        assert_eq!(addr.port(), 3000);
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    }

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
