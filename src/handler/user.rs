use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use hyper::StatusCode;
use serde_json::json;

use crate::{
    middleware::CorrelationId,
    model::{http::Response, user::User},
    service::user::{CreateUser, UpdateUser},
    state::AppState,
};

pub fn router_setup_users() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/", axum::routing::post(add_user).get(list_users))
        .route(
            "/{id}",
            axum::routing::get(get_user)
                .put(update_user)
                .delete(delete_user),
        )
}

async fn add_user(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.add_user(payload).await {
        Ok(user) => (
            StatusCode::CREATED,
            Json(json!(Response::<User> {
                correlation_id,
                message: "User created successfully".into(),
                error: "".into(),
                data: Some(user),
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

async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.list_user().await {
        Ok(users) => (
            StatusCode::OK,
            Json(json!(Response::<Vec<User>> {
                correlation_id,
                message: "Users fetched successfully".into(),
                error: "".into(),
                data: Some(users),
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
async fn get_user(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.get_user(&id).await {
        Ok(user) => (
            StatusCode::OK,
            Json(json!(Response::<User> {
                correlation_id,
                message: "User fetched successfully".into(),
                error: "".into(),
                data: Some(user),
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

async fn update_user(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateUser>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.update_user(&id, payload).await {
        Ok(user) => (
            StatusCode::OK,
            Json(json!(Response::<User> {
                correlation_id,
                message: "User updated successfully".into(),
                error: "".into(),
                data: Some(user),
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

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.service.delete_user(&id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: "User deleted successfully".into(),
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
