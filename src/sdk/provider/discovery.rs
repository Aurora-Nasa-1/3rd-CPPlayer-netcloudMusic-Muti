//! Recommendations, Discovery, Banners, Toplists, and Search Suggestions.

use super::parsers::{parse_banner, parse_playlist, parse_track};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn get_recommended_songs(client: &ApiClient, req: RecommendationsRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.recommend_songs(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let songs = res.body["data"]["dailySongs"]
        .as_array()
        .or_else(|| res.body["recommend"].as_array());

    let items = match songs {
        Some(arr) => arr.iter().map(parse_track).collect(),
        None => vec![],
    };

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_recommended_playlists(client: &ApiClient, _req: RecommendationsRequest) -> Result<PlaylistPage, Failure> {
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

pub async fn get_personalized_playlists(client: &ApiClient, req: RecommendationsRequest) -> Result<PlaylistPage, Failure> {
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

pub async fn get_top_playlists(client: &ApiClient, req: PlaylistBrowseRequest) -> Result<PlaylistPage, Failure> {
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

pub async fn get_highquality_playlists(client: &ApiClient, req: PlaylistBrowseRequest) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("cat".to_string(), req.category);
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.top_playlist_highquality(&q).await.map_err(|e| Failure {
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

pub async fn get_personalized_new_songs(client: &ApiClient, req: RecommendationsRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.personalized_newsong(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["result"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_track(&item["song"]))
                .collect()
        })
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
            let id = item["id"].as_i64().map(|n| n.to_string()).unwrap_or_default();
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

pub async fn get_similar_songs(client: &ApiClient, req: SimilarSongsRequest) -> Result<TrackPage, Failure> {
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

pub async fn get_intelligence_songs(client: &ApiClient, req: IntelligenceSongsRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("pid".to_string(), req.playlistId);

    let res = client.playmode_intelligence_list(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(|item| parse_track(&item["songInfo"])).collect())
        .unwrap_or_default();

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

pub async fn get_search_suggestions(client: &ApiClient, req: SearchSuggestionRequest) -> Result<SuggestionPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);

    let res = client.search_suggest(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["result"]["allMatch"].as_array() {
        for item in arr {
            if let Some(kw) = item["keyword"].as_str() {
                items.push(kw.to_string());
            }
        }
    }

    Ok(SuggestionPage { items })
}

pub async fn get_recommendation_history(client: &ApiClient, _req: Bytes) -> Result<RecommendationHistory, Failure> {
    let q = Query::new();
    let res = client.history_recommend_songs(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let dates = res.body["data"]["dates"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    Ok(RecommendationHistory { dates })
}

pub async fn get_recommendation_history_tracks(client: &ApiClient, req: HistoryTracksRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("date".to_string(), req.date);

    let res = client.history_recommend_songs_detail(&q).await.map_err(|e| Failure {
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
