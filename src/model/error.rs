use axum::http::StatusCode;

pub enum AppErrorCode {
    NotFound,
    InvalidInput,
    InternalError(String),
}

pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
}

impl AppError {
    pub fn get_http_status(&self) -> StatusCode {
        match self.code {
            AppErrorCode::NotFound => StatusCode::NOT_FOUND,
            AppErrorCode::InvalidInput => StatusCode::BAD_REQUEST,
            AppErrorCode::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn get_message(&self) -> String {
        self.message.clone()
    }

    pub fn get_error(&self) -> String {
        match &self.code {
            AppErrorCode::InternalError(e) => e.into(),
            _ => "".into(),
        }
    }
}
