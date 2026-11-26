use super::Query;
use crate::error::Result;
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

impl ApiClient {
    pub async fn event_search(&self, query: &Query) -> Result<ApiResponse> {
        self.request(
            "/api/event/search/list/get/v1",
            json!({
                "keyword": query.get_or("keyword", ""),
                "limit": query.get_or("limit", "30").parse::<i32>().unwrap_or(30),
                "offset": query.get_or("offset", "0").parse::<i64>().unwrap_or(0),
            }),
            query.to_option(CryptoType::Weapi),
        )
        .await
    }
}
