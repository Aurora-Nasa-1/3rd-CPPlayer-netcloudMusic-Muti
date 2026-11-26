use super::Query;
use crate::error::Result;
/// 私人 FM 模式
/// 对应 Node.js module/personal_fm_mode.js
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

impl ApiClient {
    /// 私人 FM 模式
    /// 对应 /personal/fm/mode
    pub async fn personal_fm_mode(&self, query: &Query) -> Result<ApiResponse> {
        let mut data = json!({
            "mode": query.get_or("mode", "DEFAULT"),
            "limit": query.get_i64("limit", 3).max(1)
        });
        if let Some(sub_mode) = query
            .get("submode")
            .or_else(|| query.get("subMode"))
            .filter(|value| !value.is_empty())
        {
            data["subMode"] = json!(sub_mode);
        }
        self.request(
            "/api/v1/radio/get",
            data,
            query.to_option(CryptoType::Eapi),
        )
        .await
    }
}
