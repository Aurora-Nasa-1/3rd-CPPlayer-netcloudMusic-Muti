use super::Query;
use crate::error::Result;
/// 楼层评论
/// 对应 Node.js module/comment_floor.js
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

impl ApiClient {
    /// 楼层评论
    /// 对应 /comment/floor
    pub async fn comment_floor(&self, query: &Query) -> Result<ApiResponse> {
        let resource_type = query.get_or("type", "0");
        let thread_id = query.comment_thread_id(&resource_type);
        let data = json!({
            "parentCommentId": query.get_or("parentCommentId", "0"),
            "threadId": thread_id,
            "time": query.get_or("time", "-1").parse::<i64>().unwrap_or(-1),
            "limit": query.get_or("limit", "20").parse::<i64>().unwrap_or(20)
        });
        self.request(
            "/api/resource/comment/floor/get",
            data,
            query.to_option(CryptoType::Weapi),
        )
        .await
    }
}
