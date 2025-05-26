use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use hyper::StatusCode;
use serde_json::json;

use crate::{middleware::CorrelationId, model::http::Response, state::AppState};

pub fn router_setup_users() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/", axum::routing::get(test_trx))
}

async fn test_trx(
    State(state): State<Arc<AppState>>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> (StatusCode, Json<serde_json::Value>) {
    // let tx = state.db_pool.begin().await;
    // if tx.is_err() {
    //     return (
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         Json(json!(Response::<serde_json::Value> {
    //             correlation_id,
    //             message: "Failed to begin transaction".into(),
    //             error: "Internal server error".into(),
    //             data: None,
    //         })),
    //     );
    // }

    // let user_repo = Arc::new(PostgresUserRepository::new(tx));

    match state.service.user.test_trx().await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!(Response::<serde_json::Value> {
                correlation_id,
                message: "ok".into(),
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
