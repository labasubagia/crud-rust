use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    routing::{get, post, put},
};
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

const HEADER_CORRELATION_ID: &str = "X-Correlation-ID";

struct Config {
    app_name: String,

    host: IpAddr,
    port: u16,
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

#[derive(Serialize, Clone)]
struct Item {
    id: String,
    name: String,
}

struct AppState {
    config: Config,
    items: Mutex<Vec<Item>>,
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

    let app = axum::Router::new()
        .route("/", get(handler_index))
        .route("/api/healthcheck", get(handler_healthcheck))
        .route(
            "/api/items",
            post(handler_create_item).get(handler_list_items),
        )
        .route(
            "/api/items/{id}",
            put(handler_update_item)
                .get(handler_get_item)
                .delete(handler_delete_item),
        )
        .with_state(app_state.clone());

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

#[derive(Serialize, Deserialize)]
struct Response<T> {
    correlation_id: String,
    message: String,
    error: String,
    data: Option<T>,
}

async fn handler_index(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<serde_json::Value>) {
    let default_correlation_id = Uuid::new_v4().to_string();
    let correlation_id = headers
        .get(HEADER_CORRELATION_ID)
        .map(|v| v.to_str().unwrap_or(&default_correlation_id))
        .unwrap_or(&default_correlation_id);
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
            correlation_id: correlation_id.into(),
            message: format!("Welcome to {}!", app_state.config.app_name),
            error: "".into(),
            data: None,
        })),
    )
}

async fn handler_healthcheck(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<serde_json::Value>) {
    let default_correlation_id = Uuid::new_v4().to_string();
    let correlation_id = headers
        .get(HEADER_CORRELATION_ID)
        .map(|v| v.to_str().unwrap_or(&default_correlation_id))
        .unwrap_or(&default_correlation_id);
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
            correlation_id: correlation_id.into(),
            message: "ok".into(),
            error: "".into(),
            data: None,
        })),
    )
}

#[derive(Serialize, Deserialize)]
struct CreateItem {
    name: String,
}

async fn handler_list_items(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<serde_json::Value>) {
    let default_correlation_id = Uuid::new_v4().to_string();
    let correlation_id = headers
        .get(HEADER_CORRELATION_ID)
        .map(|v| v.to_str().unwrap_or(&default_correlation_id))
        .unwrap_or(&default_correlation_id);

    let items = match app_state.items.lock() {
        Ok(items) => items.to_vec(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(Response::<serde_json::Value> {
                    correlation_id: correlation_id.into(),
                    message: "failed to aqcuire lock".into(),
                    error: format!("cannot aqcuire lock: {}", e),
                    data: None,
                })),
            );
        }
    };

    (
        StatusCode::OK,
        Json(json!(Response::<Vec<Item>> {
            correlation_id: "".into(),
            message: "ok".into(),
            error: "".into(),
            data: Some(items),
        })),
    )
}

async fn handler_create_item(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateItem>,
) -> (StatusCode, Json<serde_json::Value>) {
    let default_correlation_id = Uuid::new_v4().to_string();
    let correlation_id = headers
        .get(HEADER_CORRELATION_ID)
        .map(|v| v.to_str().unwrap_or(&default_correlation_id))
        .unwrap_or(&default_correlation_id);

    let name = payload.name.trim().to_lowercase();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!(Response::<serde_json::Value> {
                correlation_id: correlation_id.into(),
                message: "Item name cannot be empty".into(),
                error: "Invalid input".into(),
                data: None,
            })),
        );
    }

    match app_state.items.lock() {
        Ok(mut items) => {
            let cur = items.iter().find(|item| item.name == name);
            match cur {
                Some(item) => (
                    StatusCode::OK,
                    Json(json!(Response::<Item> {
                        correlation_id: correlation_id.into(),
                        message: format!("Item '{}' already exists", payload.name),
                        error: "".into(),
                        data: Some(item.clone()),
                    })),
                ),
                None => {
                    let new_item = Item {
                        id: Uuid::new_v4().to_string(),
                        name,
                    };
                    items.push(new_item.clone());
                    (
                        StatusCode::CREATED,
                        Json(json!(Response::<Item> {
                            correlation_id: correlation_id.into(),
                            message: format!("Created item '{}'", payload.name),
                            error: "".into(),
                            data: Some(new_item),
                        })),
                    )
                }
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(Response::<serde_json::Value> {
                correlation_id: correlation_id.into(),
                message: "Failed to process request".into(),
                error: format!("cannot aqcuire lock: {}", e),
                data: None,
            })),
        ),
    }
}

async fn handler_get_item(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let default_correlation_id = Uuid::new_v4().to_string();
    let correlation_id = headers
        .get(HEADER_CORRELATION_ID)
        .map(|v| v.to_str().unwrap_or(&default_correlation_id))
        .unwrap_or(&default_correlation_id);

    match app_state.items.lock() {
        Ok(items) => {
            let item = items.iter().find(|item| item.id == id).cloned();
            match item {
                Some(item) => (
                    StatusCode::OK,
                    Json(json!(Response::<Item> {
                        correlation_id: correlation_id.into(),
                        message: "ok".into(),
                        error: "".into(),
                        data: Some(item),
                    })),
                ),
                None => (
                    StatusCode::NOT_FOUND,
                    Json(json!(Response::<Item> {
                        correlation_id: correlation_id.into(),
                        message: format!("item with id {} not found", id),
                        error: "".into(),
                        data: None,
                    })),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(Response::<serde_json::Value> {
                correlation_id: correlation_id.into(),
                message: "something went wrong".into(),
                error: format!("cannot aqcuire lock: {}", e),
                data: None,
            })),
        ),
    }
}

#[derive(Serialize, Deserialize)]
struct UpdateItem {
    name: String,
}

async fn handler_update_item(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateItem>,
) -> (StatusCode, Json<serde_json::Value>) {
    let default_correlation_id = Uuid::new_v4().to_string();
    let correlation_id = headers
        .get(HEADER_CORRELATION_ID)
        .map(|v| v.to_str().unwrap_or(&default_correlation_id))
        .unwrap_or(&default_correlation_id);

    let name = payload.name.trim().to_lowercase();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!(Response::<serde_json::Value> {
                correlation_id: correlation_id.into(),
                message: "Item name cannot be empty".into(),
                error: "Invalid input".into(),
                data: None,
            })),
        );
    }

    match app_state.items.lock() {
        Ok(mut items) => {
            let index = match items.iter().position(|item| item.id == id) {
                Some(index) => index,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(json!(Response::<serde_json::Value> {
                            correlation_id: correlation_id.into(),
                            message: format!("item with id {} not found", id),
                            error: "".into(),
                            data: None,
                        })),
                    );
                }
            };
            let cur = match items.get(index) {
                Some(item) => item.clone(),
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(json!(Response::<serde_json::Value> {
                            correlation_id: correlation_id.into(),
                            message: format!("item with id {} not found", id),
                            error: "".into(),
                            data: None,
                        })),
                    );
                }
            };

            let updated_item = Item { name, ..cur };
            items[index] = updated_item.clone();
            (
                StatusCode::OK,
                Json(json!(Response::<Item> {
                    correlation_id: correlation_id.into(),
                    message: format!("Updated item '{}' with id {}", payload.name, id),
                    error: "".into(),
                    data: Some(updated_item),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(Response::<serde_json::Value> {
                correlation_id: correlation_id.into(),
                message: "something went wrong".into(),
                error: format!("cannot aqcuire lock: {}", e),
                data: None,
            })),
        ),
    }
}

async fn handler_delete_item(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let default_correlation_id = Uuid::new_v4().to_string();
    let correlation_id = headers
        .get(HEADER_CORRELATION_ID)
        .map(|v| v.to_str().unwrap_or(&default_correlation_id))
        .unwrap_or(&default_correlation_id);

    match app_state.items.lock() {
        Ok(mut items) => {
            *items = items
                .clone()
                .into_iter()
                .filter(|item| item.id != id)
                .collect();
            (
                StatusCode::OK,
                Json(json!(Response::<serde_json::Value> {
                    correlation_id: correlation_id.into(),
                    message: format!("Deleted item with id {}", id),
                    error: "".into(),
                    data: None,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(Response::<serde_json::Value> {
                correlation_id: correlation_id.into(),
                message: "something went wrong".into(),
                error: format!("cannot aqcuire lock: {}", e),
                data: None,
            })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::Request, routing::delete};
    use tower::ServiceExt;
    use uuid::Uuid;

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

        let app = axum::Router::new()
            .route("/", get(handler_index))
            .with_state(app_state);

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

        let app = axum::Router::new()
            .route("/api/healthcheck", get(handler_healthcheck))
            .with_state(app_state);

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

    #[tokio::test]
    async fn test_create_item_handler() {
        let app_state = Arc::new(AppState::default());

        let app = axum::Router::new()
            .route("/api/items", post(handler_create_item))
            .with_state(app_state.clone());

        let new_item = CreateItem {
            name: "test item".into(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/items")
                    .header("Content-Type", "application/json")
                    .header(HEADER_CORRELATION_ID, Uuid::new_v4().to_string())
                    .body(axum::body::Body::from(
                        serde_json::to_string(&new_item).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["message"], format!("Created item '{}'", new_item.name));
        assert_eq!(json["error"], "");
        assert_eq!(json["data"]["name"], new_item.name);
        assert_eq!((&app_state).items.lock().unwrap().len(), 1);
        assert_eq!((&app_state).items.lock().unwrap()[0].name, new_item.name);
    }

    #[tokio::test]
    async fn test_list_items_handler() {
        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![
                Item {
                    id: "1".into(),
                    name: "item 1".into(),
                },
                Item {
                    id: "2".into(),
                    name: "item 2".into(),
                },
            ]),
        });

        let app = axum::Router::new()
            .route("/api/items", get(handler_list_items))
            .with_state(app_state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/items")
                    .header(HEADER_CORRELATION_ID, Uuid::new_v4().to_string())
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
        assert_eq!(
            json["data"].as_array().unwrap().len(),
            app_state.items.lock().unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_get_item_handler_ok() {
        let item_id = "1";

        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "test item".into(),
            }]),
        });

        let app = axum::Router::new()
            .route("/api/items/{id}", get(handler_get_item))
            .with_state(app_state.clone());

        // Test successful get
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/items/{}", item_id))
                    .header(HEADER_CORRELATION_ID, Uuid::new_v4().to_string())
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
        assert_eq!(json["data"]["id"], item_id);
        assert_eq!(json["data"]["name"], "test item");
    }

    #[tokio::test]
    async fn test_get_item_handler_not_found() {
        let item_id = "nonexistent_id";

        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![Item {
                id: "1".into(),
                name: "test item".into(),
            }]),
        });

        let app = axum::Router::new()
            .route("/api/items/{id}", get(handler_get_item))
            .with_state(app_state.clone());

        // Test item not found
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/items/{}", item_id))
                    .header(HEADER_CORRELATION_ID, Uuid::new_v4().to_string())
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json["message"],
            format!("item with id {} not found", item_id)
        );
        assert_eq!(json["error"], "");
        assert_eq!(json["data"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_update_item_handler() {
        let item_id = "1";

        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "old item".into(),
            }]),
        });

        let app = axum::Router::new()
            .route("/api/items/{id}", axum::routing::put(handler_update_item))
            .with_state(app_state.clone());

        let updated_item = UpdateItem {
            name: "updated item".into(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/items/{}", item_id))
                    .header("Content-Type", "application/json")
                    .header(HEADER_CORRELATION_ID, Uuid::new_v4().to_string())
                    .body(axum::body::Body::from(
                        serde_json::to_string(&updated_item).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json["message"],
            format!("Updated item '{}' with id {}", updated_item.name, item_id)
        );
        assert_eq!(json["error"], "");
        assert_eq!(json["data"]["id"], item_id);
        assert_eq!(json["data"]["name"], updated_item.name);
        assert_eq!((&app_state).items.lock().unwrap().len(), 1);
        assert_eq!(
            (&app_state).items.lock().unwrap()[0].name,
            updated_item.name
        );
    }

    #[tokio::test]
    async fn test_handler_delete_item() {
        let item_id = "1";

        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "test item".into(),
            }]),
        });

        let app = axum::Router::new()
            .route("/api/items/{id}", delete(handler_delete_item))
            .with_state(app_state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/items/{}", item_id))
                    .header(HEADER_CORRELATION_ID, Uuid::new_v4().to_string())
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

        assert_eq!(json["message"], format!("Deleted item with id {}", item_id));
        assert_eq!(json["error"], "");
        assert_eq!(json["data"], serde_json::Value::Null);
        assert_eq!((&app_state).items.lock().unwrap().len(), 0);
    }
}
