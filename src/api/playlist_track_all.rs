use super::Query;
use crate::error::Result;
/// 歌单所有歌曲
/// 对应 Node.js module/playlist_track_all.js
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

/// 一页歌单曲目，外加同一次歌单详情里的总数与是否还有下一页。
pub struct PlaylistTrackPage {
    pub response: ApiResponse,
    /// 歌单 `trackIds` 总数（不是本页条数）。
    pub total: i64,
    pub has_more: bool,
}

/// 把请求的 `offset` / `limit` 收敛到 `[0, total]`。
///
/// 上游的 `trackIds` 可能比 `trackCount` 短（歌曲被删除/下架），调用方也可能
/// 因为分页状态错位而请求越界的 offset。`&track_ids[start..end]` 在
/// `start > len` 时会 panic，C ABI 只能把它报成笼统的 `InvalidResponse` ——
/// 歌单其实好好的，用户却看到音源坏了。
pub fn page_bounds(total: usize, offset: i64, limit: i64) -> (usize, usize) {
    let start = (offset.max(0) as usize).min(total);
    let end = start.saturating_add(limit.max(0) as usize).min(total);
    (start, end)
}

impl ApiClient {
    /// 歌单所有歌曲（两步请求：先获取 trackIds，再获取歌曲详情）
    /// 对应 /playlist/track/all
    pub async fn playlist_track_page(&self, query: &Query) -> Result<PlaylistTrackPage> {
        let id = query.get_or("id", "0");
        let limit = query.get_or("limit", "1000").parse::<i64>().unwrap_or(1000);
        let offset = query.get_or("offset", "0").parse::<i64>().unwrap_or(0);

        // Step 1: 获取歌单详情（含 trackIds）
        let data1 = json!({
            "id": id,
            "n": 100000,
            "s": query.get_or("s", "8").parse::<i64>().unwrap_or(8)
        });
        let res = self
            .request(
                "/api/v6/playlist/detail",
                data1,
                query.to_option(CryptoType::default()),
            )
            .await?;

        // Step 2: 从 trackIds 中截取分页范围
        let track_ids = match res.body["playlist"]["trackIds"].as_array() {
            Some(track_ids) => track_ids.clone(),
            None => {
                return Ok(PlaylistTrackPage {
                    response: res,
                    total: 0,
                    has_more: false,
                });
            }
        };

        let total = track_ids.len();
        let (start, end) = page_bounds(total, offset, limit);
        let sliced = &track_ids[start..end];

        let c: Vec<String> = sliced
            .iter()
            .filter_map(|t| t["id"].as_i64())
            .map(|id| format!(r#"{{"id":{}}}"#, id))
            .collect();

        let data2 = json!({
            "c": format!("[{}]", c.join(","))
        });
        let response = self
            .request(
                "/api/v3/song/detail",
                data2,
                query.to_option(CryptoType::default()),
            )
            .await?;

        Ok(PlaylistTrackPage {
            response,
            total: total as i64,
            has_more: end < total,
        })
    }

    /// 兼容旧调用：只要歌曲详情响应，不要分页元数据。
    pub async fn playlist_track_all(&self, query: &Query) -> Result<ApiResponse> {
        Ok(self.playlist_track_page(query).await?.response)
    }
}

#[cfg(test)]
mod tests {
    use super::page_bounds;

    #[test]
    fn page_bounds_clamps_past_the_end() {
        assert_eq!(page_bounds(410, 300, 300), (300, 410));
        assert_eq!(page_bounds(410, 410, 300), (410, 410));
        assert_eq!(page_bounds(410, 600, 300), (410, 410));
        assert_eq!(page_bounds(410, -5, 300), (0, 300));
        assert_eq!(page_bounds(0, 0, 300), (0, 0));
        assert_eq!(page_bounds(410, 0, 0), (0, 0));
    }
}
