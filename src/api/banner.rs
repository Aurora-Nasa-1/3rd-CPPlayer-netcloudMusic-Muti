use super::Query;
use crate::error::Result;
/// 首页 Banner
/// 对应 Node.js module/banner.js
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

impl ApiClient {
    /// 首页 Banner
    /// 对应 /banner
    pub async fn banner(&self, query: &Query) -> Result<ApiResponse> {
        let type_val = query.get_or("type", "0");
        let client_type = match type_val.as_str() {
            "0" => "pc",
            "1" => "android",
            "2" => "iphone",
            "3" => "ipad",
            _ => "pc",
        };
        let data = json!({
            "clientType": client_type
        });
        self.request(
            "/api/v2/banner/get",
            data,
            query.to_option(CryptoType::default()),
        )
        .await
    }

    /// 首页 Banner v3（新版客户端接口）
    pub async fn banner_v3(&self, query: &Query) -> Result<ApiResponse> {
        let client_type = query.get_or("clientType", "android");
        self.request(
            "/api/banner/get/v3",
            json!({ "clientType": client_type }),
            query.to_option(CryptoType::Weapi),
        )
        .await
    }
}
