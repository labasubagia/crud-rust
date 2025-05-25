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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request as HttpRequest, StatusCode},
        middleware::from_fn,
        routing::get,
    };
    use tower::ServiceExt;

    async fn handler() -> StatusCode {
        StatusCode::OK
    }

    #[tokio::test]
    async fn test_middleware_adds_correlation_id_when_not_present() {
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn(request_middleware));

        let req = HttpRequest::builder().uri("/").body(Body::empty()).unwrap();

        let res = app.oneshot(req).await.unwrap();

        assert!(res.headers().contains_key(X_CORRELATION_ID));
        let correlation_id = res.headers().get(X_CORRELATION_ID).unwrap();
        assert!(Uuid::parse_str(correlation_id.to_str().unwrap()).is_ok());
    }

    #[tokio::test]
    async fn test_middleware_preserves_existing_correlation_id() {
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn(request_middleware));

        let correlation_id = "test-correlation-id";
        let req = HttpRequest::builder()
            .uri("/")
            .header(X_CORRELATION_ID, correlation_id)
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();

        assert!(res.headers().contains_key(X_CORRELATION_ID));
        let response_correlation_id = res.headers().get(X_CORRELATION_ID).unwrap();
        assert_eq!(response_correlation_id.to_str().unwrap(), correlation_id);
    }
}
