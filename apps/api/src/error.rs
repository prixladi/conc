use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use daemon_client::ErrorResponse;
use serde_json::json;

/// Errors returned to API clients as `{"error": "<code>", "message": "<text>"}`.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "missing or invalid API key, send it as 'Authorization: Bearer <key>'",
        )
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", message)
    }
}

impl From<ErrorResponse> for ApiError {
    fn from(e: ErrorResponse) -> Self {
        let message = e.to_string();
        match e {
            ErrorResponse::ProjectNotFound(_) => {
                Self::new(StatusCode::NOT_FOUND, "project_not_found", message)
            }
            ErrorResponse::ServiceNotFound(_) => {
                Self::new(StatusCode::NOT_FOUND, "service_not_found", message)
            }
            ErrorResponse::Client(_) => Self::bad_request(message),
            ErrorResponse::Socket { .. } => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "daemon_unavailable",
                message,
            ),
            ErrorResponse::Daemon(_) | ErrorResponse::Malformed(_) => {
                Self::new(StatusCode::BAD_GATEWAY, "daemon_error", message)
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            tracing::error!(code = self.code, "{}", self.message);
        }
        let body = Json(json!({ "error": self.code, "message": self.message }));
        if self.status == StatusCode::UNAUTHORIZED {
            return (self.status, [(header::WWW_AUTHENTICATE, "Bearer")], body).into_response();
        }
        (self.status, body).into_response()
    }
}

/// `Query` extractor whose rejection uses the API's JSON error shape.
pub struct ApiQuery<T>(pub T);

impl<T, S> axum::extract::FromRequestParts<S> for ApiQuery<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        axum::extract::Query::<T>::from_request_parts(parts, state)
            .await
            .map(|axum::extract::Query(v)| ApiQuery(v))
            .map_err(|e| ApiError::bad_request(e.body_text()))
    }
}
