use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

pub const X_CORRELATION_ID: &str = "X-Correlation-Id";

pub type CorrelationId = String;

pub async fn request_middleware(mut request: Request, next: Next) -> Response {
    let correlation_id: CorrelationId = request
        .headers()
        .get(X_CORRELATION_ID)
        .and_then(|h| h.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.extensions_mut().insert(correlation_id.clone());
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        X_CORRELATION_ID,
        HeaderValue::from_str(&correlation_id).unwrap(),
    );
    response
}
