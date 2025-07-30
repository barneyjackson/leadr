use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub current_cursor: Option<String>,
    pub total_returned: usize,
    pub page_size: usize,
}