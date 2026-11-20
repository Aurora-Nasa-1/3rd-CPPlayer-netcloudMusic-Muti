use super::Query;
use crate::error::Result;
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

impl ApiClient {
    pub async fn song_url_v1(&self, query: &Query) -> Result<ApiResponse> {
        let id = query.get_or("id", "0");
        let requested_level = query.get_or("level", "standard");

        let levels = [
            "sky", "jymaster", "vivid", "jyeffect", "hires", "lossless", "exhigh", "higher",
            "standard",
        ];
        let start_index = levels
            .iter()
            .position(|&l| l == requested_level)
            .unwrap_or(levels.len() - 1);

        let mut final_response = None;

        for &level in &levels[start_index..] {
            let mut data = json!({
                "ids": format!("[{}]", id),
                "level": level,
                "encodeType": "flac"
            });
            if level == "sky" {
                data["immerseType"] = json!(query.get_or("immerseType", "c51"));
            }
            if level == "vivid" {
                data["encodeType"] = json!("mp3");
            }

            if let Ok(response) = self
                .request(
                    "/api/song/enhance/player/url/v1",
                    data,
                    query.to_option(CryptoType::Xeapi),
                )
                .await
            {
                if let Some(url) = response
                    .body
                    .get("data")
                    .and_then(|d| d.get(0))
                    .and_then(|item| item.get("url"))
                    .and_then(|u| u.as_str())
                {
                    if !url.is_empty() && url != "null" {
                        return Ok(response);
                    }
                }
                final_response = Some(response);
            }
        }

        match final_response {
            Some(resp) => Ok(resp),
            None => Err(crate::error::NcmError::Unknown(
                "No audio URL found after quality fallback".to_string(),
            )),
        }
    }
}
