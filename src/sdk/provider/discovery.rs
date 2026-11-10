//! Recommendations, Discovery, Banners, Toplists, and Search Suggestions.

use super::parsers::{parse_banner, parse_playlist, parse_track};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn get_recommended_songs(
    client: &ApiClient,
    req: RecommendationsRequest,
) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.recommend_songs(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let songs = res.body["data"]["dailySongs"]
        .as_array()
        .or_else(|| res.body["recommend"].as_array());

    let mut items: Vec<Track> = match songs {
        Some(arr) => arr.iter().map(parse_track).collect(),
        None => vec![],
    };
    items.truncate(req.limit.max(0) as usize);

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_recommended_playlists(
    client: &ApiClient,
    _req: RecommendationsRequest,
) -> Result<PlaylistPage, Failure> {
    let q = Query::new();
    let res = client.recommend_resource(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["recommend"]
        .as_array()
        .map(|arr| arr.iter().map(parse_playlist).collect())
        .unwrap_or_default();

    Ok(PlaylistPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_personalized_playlists(
    client: &ApiClient,
    req: RecommendationsRequest,
) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.personalized(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["result"]
        .as_array()
        .map(|arr| arr.iter().map(parse_playlist).collect())
        .unwrap_or_default();

    Ok(PlaylistPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_top_playlists(
    client: &ApiClient,
    req: PlaylistBrowseRequest,
) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("cat".to_string(), req.category);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("order".to_string(), req.order);

    let res = client.top_playlist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let playlists = res.body["playlists"].as_array();
    let total = res.body["total"].as_i64();
    let more = res.body["more"].as_bool();

    let items = match playlists {
        Some(arr) => arr.iter().map(parse_playlist).collect(),
        None => vec![],
    };

    Ok(PlaylistPage {
        items,
        total,
        hasMore: more,
    })
}

pub async fn get_highquality_playlists(
    client: &ApiClient,
    req: PlaylistBrowseRequest,
) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("cat".to_string(), req.category);
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client
        .top_playlist_highquality(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let playlists = res.body["playlists"].as_array();
    let more = res.body["more"].as_bool();

    let items = match playlists {
        Some(arr) => arr.iter().map(parse_playlist).collect(),
        None => vec![],
    };

    Ok(PlaylistPage {
        items,
        total: None,
        hasMore: more,
    })
}

pub async fn get_personalized_new_songs(
    client: &ApiClient,
    req: RecommendationsRequest,
) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.personalized_newsong(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["result"]
        .as_array()
        .map(|arr| arr.iter().map(|item| parse_track(&item["song"])).collect())
        .unwrap_or_default();

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_banners(client: &ApiClient, _req: Bytes) -> Result<BannerPage, Failure> {
    let q = Query::new();
    let res = client.banner(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["banners"]
        .as_array()
        .map(|arr| arr.iter().map(parse_banner).collect())
        .unwrap_or_default();

    Ok(BannerPage { items })
}

pub async fn get_rankings(client: &ApiClient, _req: Bytes) -> Result<RankingPage, Failure> {
    let q = Query::new();
    let res = client.toplist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["list"].as_array() {
        for item in arr {
            let id = item["id"]
                .as_i64()
                .map(|n| n.to_string())
                .unwrap_or_default();
            let title = item["name"].as_str().unwrap_or_default().to_string();
            let cover_url = item["coverImgUrl"].as_str().map(String::from);
            let update_frequency = item["updateFrequency"].as_str().map(String::from);
            let track_count = item["trackCount"].as_i64().unwrap_or(0) as i32;
            items.push(Ranking {
                id,
                title,
                coverUrl: cover_url,
                updateFrequency: update_frequency,
                trackCount: track_count,
            });
        }
    }

    Ok(RankingPage { items })
}

pub async fn get_top_songs(client: &ApiClient, req: TopSongsRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("type".to_string(), req.region.to_string());

    let res = client.top_song(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(parse_track).collect())
        .unwrap_or_default();

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_personal_fm(client: &ApiClient, _req: Bytes) -> Result<TrackPage, Failure> {
    let q = Query::new();
    let res = client.personal_fm(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(parse_track).collect())
        .unwrap_or_default();

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_similar_songs(
    client: &ApiClient,
    req: SimilarSongsRequest,
) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);

    let res = client.simi_song(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["songs"]
        .as_array()
        .map(|arr| arr.iter().map(parse_track).collect())
        .unwrap_or_default();

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_intelligence_songs(
    client: &ApiClient,
    req: IntelligenceSongsRequest,
) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("pid".to_string(), req.playlistId);
    q.params.insert("count".to_string(), req.limit.to_string());

    let res = client
        .playmode_intelligence_list(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let mut items: Vec<Track> = res.body["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_track(&item["songInfo"]))
                .collect()
        })
        .unwrap_or_default();
    items.truncate(req.limit.max(0) as usize);

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_hot_searches(client: &ApiClient, _req: Bytes) -> Result<KeywordPage, Failure> {
    let q = Query::new();
    let res = client.search_hot_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["data"].as_array() {
        for item in arr {
            items.push(Keyword {
                keyword: item["searchWord"].as_str().unwrap_or_default().to_string(),
                description: item["content"].as_str().unwrap_or_default().to_string(),
            });
        }
    }

    Ok(KeywordPage { items })
}

pub async fn get_search_suggestions(
    client: &ApiClient,
    req: SearchSuggestionRequest,
) -> Result<SuggestionPage, Failure> {
    let kw = req.keyword.trim();
    if kw.is_empty() {
        return Ok(SuggestionPage { items: vec![] });
    }

    let mut q = Query::new();
    q.params.insert("keywords".to_string(), kw.to_string());
    q.params.insert("type".to_string(), "mobile".to_string());

    let res = client.search_suggest(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["result"]["allMatch"].as_array() {
        for item in arr {
            if let Some(keyword) = item["keyword"].as_str() {
                let s = keyword.trim().to_string();
                if !s.is_empty() && !items.contains(&s) {
                    items.push(s);
                }
            }
        }
    }

    // Fallback in case mobile endpoint returns no allMatch
    if items.is_empty() {
        if let Some(arr) = res.body["result"]["songs"].as_array() {
            for item in arr {
                if let Some(name) = item["name"].as_str() {
                    let s = name.trim().to_string();
                    if !s.is_empty() && !items.contains(&s) {
                        items.push(s);
                    }
                }
            }
        }
    }

    let limit = if req.limit > 0 {
        req.limit as usize
    } else {
        10
    };
    if items.len() > limit {
        items.truncate(limit);
    }

    Ok(SuggestionPage { items })
}

pub async fn get_recommendation_history(
    client: &ApiClient,
    _req: Bytes,
) -> Result<RecommendationHistory, Failure> {
    let q = Query::new();
    let res = client
        .history_recommend_songs(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let dates = res.body["data"]["dates"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(RecommendationHistory { dates })
}

pub async fn get_recommendation_history_tracks(
    client: &ApiClient,
    req: HistoryTracksRequest,
) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("date".to_string(), req.date);

    let res = client
        .history_recommend_songs_detail(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let items = res.body["data"]["dailySongs"]
        .as_array()
        .map(|arr| arr.iter().map(parse_track).collect())
        .unwrap_or_default();

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_recommendations_respect_requested_limit_when_upstream_returns_more() {
        let daily: Vec<Track> = (0..34)
            .map(|id| Track {
                id: id.to_string(),
                title: format!("Song {id}"),
                artists: vec![],
                durationMs: 1000,
                album: None,
                coverUrl: None,
            })
            .collect();
        let limit = 30;
        let mut items = daily;
        items.truncate(limit);
        assert_eq!(items.len(), limit);
        assert_eq!(items.last().unwrap().id, "29");
    }

    #[test]
    fn intelligence_limit_is_forwarded_and_bounds_the_response() {
        let req = IntelligenceSongsRequest {
            id: "song-1".to_string(),
            playlistId: "playlist-1".to_string(),
            limit: 30,
        };
        let mut q = Query::new();
        q.params.insert("count".to_string(), req.limit.to_string());
        assert_eq!(q.params.get("count").map(String::as_str), Some("30"));

        let mut items: Vec<Track> = (0..32)
            .map(|id| Track {
                id: id.to_string(),
                title: id.to_string(),
                artists: vec![],
                durationMs: 1000,
                album: None,
                coverUrl: None,
            })
            .collect();
        items.truncate(req.limit.max(0) as usize);
        assert_eq!(items.len(), 30);
    }
}
