use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

pub const X_CORRELATION_ID: &str = "X-Correlation-Id";

pub type CorrelationId = String;

pub async fn request_middleware(mut req: Request, next: Next) -> Response {
    let correlation_id: CorrelationId = req
        .headers()
        .get(X_CORRELATION_ID)
        .and_then(|h| h.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(correlation_id.clone());
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        X_CORRELATION_ID,
        HeaderValue::from_str(&correlation_id).unwrap(),
    );
    res
}
