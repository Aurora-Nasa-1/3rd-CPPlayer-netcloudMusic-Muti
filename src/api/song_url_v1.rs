use super::Query;
use crate::error::Result;
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

impl ApiClient {
    pub async fn song_url_v1(&self, query: &Query) -> Result<ApiResponse> {
        let id = query.get_or("id", "0");
        let requested_level = query.get_or("level", "standard");

        let fallback_levels: Vec<&str> = match requested_level.as_str() {
            "standard" => vec!["standard"],
            "higher" => vec!["higher", "standard"],
            "exhigh" => vec!["exhigh", "standard"],
            "lossless" => vec!["lossless", "exhigh", "standard"],
            "sky" | "jymaster" | "vivid" | "jyeffect" | "hires" => {
                vec![requested_level.as_str(), "lossless", "exhigh", "standard"]
            }
            _ => vec!["standard"],
        };

        let mut final_response = None;

        for level in fallback_levels {
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
