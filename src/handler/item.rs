use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

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
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.item.list().await {
        Ok(items) => (
            StatusCode::OK,
            Json(json!(Response::<Vec<Item>> {
                correlation_id,
                message: "ok".into(),
                error: "".into(),
                data: Some(items),
            })),
        ),
        Err(e) => (
            e.get_http_status(),
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: e.get_message(),
                error: e.get_error(),
                data: None,
            })),
        ),
    }
}

async fn create_item(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    Json(payload): Json<CreateItem>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.item.create(payload.name).await {
        Ok(item) => (
            StatusCode::CREATED,
            Json(json!(Response::<Item> {
                correlation_id,
                message: format!("Created item '{}'", item.name),
                error: "".into(),
                data: Some(item),
            })),
        ),
        Err(e) => (
            e.get_http_status(),
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: e.get_message(),
                error: e.get_error(),
                data: None,
            })),
        ),
    }
}

async fn get_item(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.item.get(id).await {
        Ok(item) => (
            StatusCode::OK,
            Json(json!(Response::<Item> {
                correlation_id,
                message: "ok".into(),
                error: "".into(),
                data: Some(item),
            })),
        ),
        Err(e) => (
            e.get_http_status(),
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: e.get_message(),
                error: e.get_error(),
                data: None,
            })),
        ),
    }
}

async fn update_item(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateItem>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.item.update(id, payload.name.clone()).await {
        Ok(item) => (
            StatusCode::OK,
            Json(json!(Response::<Item> {
                correlation_id,
                message: format!("Updated item '{}' with id {}", item.name, item.id),
                error: "".into(),
                data: Some(item),
            })),
        ),
        Err(e) => (
            e.get_http_status(),
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: e.get_message(),
                error: e.get_error(),
                data: None,
            })),
        ),
    }
}

async fn delete_item(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.item.delete(id.clone()).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: format!("Deleted item with id {}", id),
                error: "".into(),
                data: None,
            })),
        ),
        Err(e) => (
            e.get_http_status(),
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: e.get_message(),
                error: e.get_error(),
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
    use crate::repository::Repository;
    use crate::repository::item::InMemoryItemRepository;
    use crate::service::Service;
    use axum::http::Request;
    use std::sync::Mutex;
    use tower::ServiceExt;

    fn setup_app(state: Arc<AppState>) -> axum::Router {
        axum::Router::new()
            .nest("/api/items", router_setup_items())
            .layer(axum::middleware::from_fn(middleware::request_middleware))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_create_item() {
        let config = Arc::new(Config::default());

        let item_repo = Arc::new(InMemoryItemRepository::new());
        let repo = Arc::new(Repository {
            item: item_repo.clone(),
        });
        let service = Arc::new(Service::new(config.clone(), repo.clone()));
        let app_state = Arc::new(AppState {
            config: config.clone(),
            service: service,
        });
        let app = setup_app(app_state);

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
        assert_eq!((&item_repo).items.lock().unwrap().len(), 1);
        assert_eq!((&item_repo).items.lock().unwrap()[0].name, new_item.name);
    }

    #[tokio::test]
    async fn test_list_items() {
        let config = Arc::new(Config::default());
        let item_repo = Arc::new(InMemoryItemRepository {
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
        let repo = Arc::new(Repository {
            item: item_repo.clone(),
        });
        let service = Arc::new(Service::new(config.clone(), repo.clone()));
        let app_state = Arc::new(AppState {
            config: config.clone(),
            service: service,
        });
        let app = setup_app(app_state);

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
            item_repo.items.lock().unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_get_item_ok() {
        let item_id = "1";

        let config = Arc::new(Config::default());
        let item_repo = Arc::new(InMemoryItemRepository {
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "test item".into(),
            }]),
        });
        let repo = Arc::new(Repository {
            item: item_repo.clone(),
        });
        let service = Arc::new(Service::new(config.clone(), repo.clone()));
        let app_state = Arc::new(AppState {
            config: config.clone(),
            service: service,
        });
        let app = setup_app(app_state);

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

        let config = Arc::new(Config::default());
        let item_repo = Arc::new(InMemoryItemRepository {
            items: Mutex::new(vec![Item {
                id: "1".into(),
                name: "test item".into(),
            }]),
        });
        let repo = Arc::new(Repository {
            item: item_repo.clone(),
        });
        let service = Arc::new(Service::new(config.clone(), repo.clone()));
        let app_state = Arc::new(AppState {
            config: config.clone(),
            service: service,
        });
        let app = setup_app(app_state);

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
            format!("Item with id {} not found", item_id)
        );
        assert_eq!(json["error"], "");
        assert_eq!(json["data"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_update_item() {
        let item_id = "1";

        let config = Arc::new(Config::default());
        let item_repo = Arc::new(InMemoryItemRepository {
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "old item".into(),
            }]),
        });

        let repo = Arc::new(Repository {
            item: item_repo.clone(),
        });

        let service = Arc::new(Service::new(config.clone(), repo.clone()));
        let app_state = Arc::new(AppState {
            config: config.clone(),
            service: service,
        });
        let app = setup_app(app_state);

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
        assert_eq!((&item_repo).items.lock().unwrap().len(), 1);
        assert_eq!(
            (&item_repo).items.lock().unwrap()[0].name,
            updated_item.name
        );
    }

    #[tokio::test]
    async fn test_delete() {
        let item_id = "1";

        let config = Arc::new(Config::default());
        let item_repo = Arc::new(InMemoryItemRepository {
            items: Mutex::new(vec![Item {
                id: item_id.into(),
                name: "test item".into(),
            }]),
        });
        let repo = Arc::new(Repository {
            item: item_repo.clone(),
        });

        let service = Arc::new(Service::new(config.clone(), repo.clone()));
        let app_state = Arc::new(AppState {
            config: config.clone(),
            service: service,
        });
        let app = setup_app(app_state);

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
        assert_eq!((&item_repo).items.lock().unwrap().len(), 0);
    }
}
