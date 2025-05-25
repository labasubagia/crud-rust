use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub correlation_id: String,
    pub message: String,
    pub error: String,
    pub data: Option<T>,
}
