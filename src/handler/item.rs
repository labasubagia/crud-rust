use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::CorrelationId;
use crate::model::{http::Response, item::Item};
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
struct CreateItem {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateItem {
    pub name: String,
}

pub fn router_setup_items() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/", axum::routing::get(list_items).post(create_item))
        .route(
            "/{id}",
            axum::routing::get(get_item)
                .put(update_item)
                .delete(delete_item),
        )
}

async fn list_items(
    State(app_state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>) {
    let items = match app_state.items.lock() {
        Ok(items) => items.to_vec(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(Response::<serde_json::Value> {
                    correlation_id,
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

async fn create_item(
    State(app_state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    Json(payload): Json<CreateItem>,
) -> (StatusCode, Json<serde_json::Value>) {
    let name = payload.name.trim().to_lowercase();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
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
                        correlation_id,
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
                            correlation_id,
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
                correlation_id,
                message: "Failed to process request".into(),
                error: format!("cannot aqcuire lock: {}", e),
                data: None,
            })),
        ),
    }
}

async fn get_item(
    State(app_state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match app_state.items.lock() {
        Ok(items) => {
            let item = items.iter().find(|item| item.id == id).cloned();
            match item {
                Some(item) => (
                    StatusCode::OK,
                    Json(json!(Response::<Item> {
                        correlation_id,
                        message: "ok".into(),
                        error: "".into(),
                        data: Some(item),
                    })),
                ),
                None => (
                    StatusCode::NOT_FOUND,
                    Json(json!(Response::<Item> {
                        correlation_id,
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
                correlation_id,
                message: "something went wrong".into(),
                error: format!("cannot aqcuire lock: {}", e),
                data: None,
            })),
        ),
    }
}

async fn update_item(
    State(app_state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateItem>,
) -> (StatusCode, Json<serde_json::Value>) {
    let name = payload.name.trim().to_lowercase();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
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
                            correlation_id,
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
                            correlation_id,
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
                    correlation_id,
                    message: format!("Updated item '{}' with id {}", payload.name, id),
                    error: "".into(),
                    data: Some(updated_item),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: "something went wrong".into(),
                error: format!("cannot aqcuire lock: {}", e),
                data: None,
            })),
        ),
    }
}

async fn delete_item(
    State(app_state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
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
                    correlation_id,
                    message: format!("Deleted item with id {}", id),
                    error: "".into(),
                    data: None,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
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
    use crate::config::Config;
    use crate::middleware;
    use axum::http::Request;
    use std::sync::Mutex;
    use tower::ServiceExt;

    fn setup_app(app_state: Arc<AppState>) -> axum::Router {
        axum::Router::new()
            .nest("/api/items", router_setup_items())
            .layer(axum::middleware::from_fn(middleware::request_middleware))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_create_item() {
        let app_state = Arc::new(AppState::default());
        let app = setup_app(app_state.clone());

        let new_item = CreateItem {
            name: "test item".into(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/items")
                    .header("Content-Type", "application/json")
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
    async fn test_list_items() {
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
        let app = setup_app(app_state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/items")
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
    async fn test_get_item_ok() {
        let item_id = "1";

        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "test item".into(),
            }]),
        });
        let app = setup_app(app_state.clone());

        // Test successful get
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/items/{}", item_id))
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
    async fn test_get_item_not_found() {
        let item_id = "nonexistent_id";

        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![Item {
                id: "1".into(),
                name: "test item".into(),
            }]),
        });
        let app = setup_app(app_state.clone());

        // Test item not found
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/items/{}", item_id))
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
    async fn test_update_item() {
        let item_id = "1";

        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "old item".into(),
            }]),
        });
        let app = setup_app(app_state.clone());

        let updated_item = UpdateItem {
            name: "updated item".into(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/items/{}", item_id))
                    .header("Content-Type", "application/json")
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
    async fn test_delete() {
        let item_id = "1";

        let app_state = Arc::new(AppState {
            config: Config::default(),
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "test item".into(),
            }]),
        });
        let app = setup_app(app_state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/items/{}", item_id))
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
