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
    match state.service.list_item().await {
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
    match state.service.create_item(payload.name).await {
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
    match state.service.get_item(id).await {
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
    match state.service.update_item(id, payload.name.clone()).await {
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
    match state.service.delete_item(id.clone()).await {
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
