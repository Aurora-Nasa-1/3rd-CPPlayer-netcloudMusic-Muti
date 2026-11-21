use super::Query;
use crate::error::Result;
/// 每日推荐歌曲
/// 对应 Node.js module/recommend_songs.js
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

impl ApiClient {
    /// 每日推荐歌曲（需要登录）
    /// 对应 /recommend/songs
    pub async fn recommend_songs(&self, query: &Query) -> Result<ApiResponse> {
        let mut data = json!({
            "ispush": query.get_or("ispush", "false"),
        });
        if let Some(afresh) = query.get("afresh") {
            data["afresh"] = json!(afresh);
        }
        self.request(
            "/api/v3/discovery/recommend/songs",
            data,
            query.to_option(CryptoType::Xeapi),
        )
        .await
    }

    /// 每日风格推荐歌曲（新版首页推荐）
    pub async fn recommend_category_songs(&self, query: &Query) -> Result<ApiResponse> {
        self.request(
            "/api/homepage/category/daily/song/list",
            json!({}),
            query.to_option(CryptoType::Xeapi),
        )
        .await
    }

    /// 每日风格推荐配置
    pub async fn recommend_category_configs(&self, query: &Query) -> Result<ApiResponse> {
        self.request(
            "/api/homepage/daily/song/config/get",
            json!({}),
            query.to_option(CryptoType::Xeapi),
        )
        .await
    }
}
