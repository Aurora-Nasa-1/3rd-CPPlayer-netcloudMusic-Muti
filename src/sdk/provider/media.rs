//! Videos, Music Videos (MVs), DJ Radios, Programs, and Media Categories.

use super::parsers::{parse_dj_program, parse_dj_radio, parse_social_user, parse_video_item};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn get_mv(client: &ApiClient, req: IdRequest) -> Result<VideoItem, Failure> {
    let mut q = Query::new();
    q.params.insert("mvid".to_string(), req.id.clone());

    let res = client.mv_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["data"].is_object() {
        Ok(parse_video_item(&res.body["data"]))
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("MV {} not found", req.id),
        })
    }
}

pub async fn resolve_mv(
    client: &ApiClient,
    req: VideoResolveRequest,
) -> Result<MediaResource, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("r".to_string(), req.resolution.to_string());

    let res = client.mv_url(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let url = res.body["data"]["url"].as_str().unwrap_or("").to_string();
    if url.is_empty() {
        return Err(Failure {
            category: "NotFound".to_string(),
            message: format!("No video URL found for MV {}", req.id),
        });
    }

    Ok(MediaResource {
        url,
        actualQuality: req.resolution.to_string(),
        headers: vec![],
        expiresAtEpochMs: None,
        format: Some("mp4".to_string()),
        bitrate: res.body["data"]["r"].as_i64().map(|n| n as i32),
        sizeBytes: res.body["data"]["size"].as_i64(),
    })
}

pub async fn get_all_mvs(
    client: &ApiClient,
    req: VideoBrowseRequest,
) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params.insert("area".to_string(), req.area);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.mv_all(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(parse_video_item).collect())
        .unwrap_or_default();

    let total = res.body["count"].as_i64();

    Ok(VideoPage {
        items,
        total,
        hasMore: None,
    })
}

pub async fn get_latest_mvs(client: &ApiClient, req: PageRequest) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.mv_first(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(parse_video_item).collect())
        .unwrap_or_default();

    Ok(VideoPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn subscribe_mv(
    client: &ApiClient,
    req: SubscriptionRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("mvid".to_string(), req.id.clone());
    q.params.insert(
        "t".to_string(),
        if req.subscribed { "1" } else { "0" }.to_string(),
    );

    let res = client.mv_sub(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_subscribed_mvs(
    client: &ApiClient,
    req: PageRequest,
) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.mv_sublist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(parse_video_item).collect())
        .unwrap_or_default();

    let total = res.body["count"].as_i64();

    Ok(VideoPage {
        items,
        total,
        hasMore: None,
    })
}

pub async fn get_video(client: &ApiClient, req: IdRequest) -> Result<VideoItem, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.video_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["data"].is_object() {
        Ok(parse_video_item(&res.body["data"]))
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("Video {} not found", req.id),
        })
    }
}

pub async fn resolve_video(
    client: &ApiClient,
    req: VideoResolveRequest,
) -> Result<MediaResource, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("r".to_string(), req.resolution.to_string());

    let res = client.video_url(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let url = res.body["urls"][0]["url"]
        .as_str()
        .unwrap_or("")
        .to_string();
    if url.is_empty() {
        return Err(Failure {
            category: "NotFound".to_string(),
            message: format!("No playable URL for video {}", req.id),
        });
    }

    Ok(MediaResource {
        url,
        actualQuality: req.resolution.to_string(),
        headers: vec![],
        expiresAtEpochMs: None,
        format: Some("mp4".to_string()),
        bitrate: res.body["urls"][0]["r"].as_i64().map(|n| n as i32),
        sizeBytes: res.body["urls"][0]["size"].as_i64(),
    })
}

pub async fn get_video_groups(client: &ApiClient, _req: Bytes) -> Result<CategoryPage, Failure> {
    let q = Query::new();
    let res = client.video_group_list(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["data"].as_array() {
        for v in arr {
            items.push(Category {
                id: v["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                name: v["name"].as_str().unwrap_or_default().to_string(),
            });
        }
    }

    Ok(CategoryPage { items })
}

pub async fn get_video_timeline(
    client: &ApiClient,
    req: PageRequest,
) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.video_timeline_all(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["datas"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_video_item(&item["data"]))
                .collect()
        })
        .unwrap_or_default();

    let has_more = res.body["hasmore"].as_bool();

    Ok(VideoPage {
        items,
        total: None,
        hasMore: has_more,
    })
}

pub async fn search_mvs(client: &ApiClient, req: SearchRequest) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "1004".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["result"]["mvs"]
        .as_array()
        .map(|arr| arr.iter().map(parse_video_item).collect())
        .unwrap_or_default();

    let count = res.body["result"]["mvCount"].as_i64();

    Ok(VideoPage {
        items,
        total: count,
        hasMore: None,
    })
}

pub async fn search_videos(client: &ApiClient, req: SearchRequest) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "1014".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["result"]["videos"]
        .as_array()
        .map(|arr| arr.iter().map(parse_video_item).collect())
        .unwrap_or_default();

    let count = res.body["result"]["videoCount"].as_i64();

    Ok(VideoPage {
        items,
        total: count,
        hasMore: None,
    })
}

pub async fn get_video_categories(
    client: &ApiClient,
    _req: Bytes,
) -> Result<CategoryPage, Failure> {
    let q = Query::new();
    let res = client.video_category_list(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["data"].as_array() {
        for v in arr {
            items.push(Category {
                id: v["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                name: v["name"].as_str().unwrap_or_default().to_string(),
            });
        }
    }

    Ok(CategoryPage { items })
}

pub async fn get_videos_by_group(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params.insert("groupId".to_string(), req.id);
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.video_group(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["datas"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_video_item(&item["data"]))
                .collect()
        })
        .unwrap_or_default();

    let has_more = res.body["hasmore"].as_bool();

    Ok(VideoPage {
        items,
        total: None,
        hasMore: has_more,
    })
}

pub async fn get_recommended_video_timeline(
    client: &ApiClient,
    req: PageRequest,
) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client
        .video_timeline_recommend(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let items = res.body["datas"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_video_item(&item["data"]))
                .collect()
        })
        .unwrap_or_default();

    let has_more = res.body["hasmore"].as_bool();

    Ok(VideoPage {
        items,
        total: None,
        hasMore: has_more,
    })
}

pub async fn subscribe_video(
    client: &ApiClient,
    req: SubscriptionRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert(
        "t".to_string(),
        if req.subscribed { "1" } else { "0" }.to_string(),
    );

    let res = client.video_sub(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_artist_mvs(client: &ApiClient, req: PagedIdRequest) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.artist_mv(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["mvs"]
        .as_array()
        .map(|arr| arr.iter().map(parse_video_item).collect())
        .unwrap_or_default();

    let has_more = res.body["hasMore"].as_bool();

    Ok(VideoPage {
        items,
        total: None,
        hasMore: has_more,
    })
}

// DJ Radios
pub async fn get_dj_radio(client: &ApiClient, req: IdRequest) -> Result<DjRadio, Failure> {
    let mut q = Query::new();
    q.params.insert("rid".to_string(), req.id.clone());

    let res = client.dj_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["data"].is_object() {
        Ok(parse_dj_radio(&res.body["data"]))
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("DJ radio {} not found", req.id),
        })
    }
}

pub async fn get_dj_programs(
    client: &ApiClient,
    req: DjProgramsRequest,
) -> Result<DjProgramPage, Failure> {
    let mut q = Query::new();
    q.params.insert("rid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());
    q.params
        .insert("asc".to_string(), req.ascending.to_string());

    let res = client.dj_program(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["programs"]
        .as_array()
        .map(|arr| arr.iter().map(parse_dj_program).collect())
        .unwrap_or_default();

    let count = res.body["count"].as_i64();
    let more = res.body["more"].as_bool();

    Ok(DjProgramPage {
        items,
        total: count,
        hasMore: more,
    })
}

pub async fn get_hot_dj_radios(
    client: &ApiClient,
    req: PageRequest,
) -> Result<DjRadioPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.dj_hot(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["djRadios"]
        .as_array()
        .map(|arr| arr.iter().map(parse_dj_radio).collect())
        .unwrap_or_default();

    let count = res.body["count"].as_i64();
    let has_more = res.body["hasMore"].as_bool();

    Ok(DjRadioPage {
        items,
        total: count,
        hasMore: has_more,
    })
}

pub async fn get_dj_ranking(client: &ApiClient, req: PageRequest) -> Result<DjRadioPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.dj_program_toplist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["toplist"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_dj_radio(&item["program"]["dj"]))
                .collect()
        })
        .unwrap_or_default();

    Ok(DjRadioPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_recommended_dj_radios(
    client: &ApiClient,
    _req: PageRequest,
) -> Result<DjRadioPage, Failure> {
    let q = Query::new();
    let res = client
        .dj_personalize_recommend(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(parse_dj_radio).collect())
        .unwrap_or_default();

    Ok(DjRadioPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn subscribe_dj_radio(
    client: &ApiClient,
    req: SubscriptionRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert(
        "t".to_string(),
        if req.subscribed { "1" } else { "0" }.to_string(),
    );

    let res = client.dj_sub(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_subscribed_dj_radios(
    client: &ApiClient,
    req: PageRequest,
) -> Result<DjRadioPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.dj_sublist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["djRadios"]
        .as_array()
        .map(|arr| arr.iter().map(parse_dj_radio).collect())
        .unwrap_or_default();

    let count = res.body["count"].as_i64();
    let has_more = res.body["hasMore"].as_bool();

    Ok(DjRadioPage {
        items,
        total: count,
        hasMore: has_more,
    })
}

pub async fn get_recommended_programs(
    client: &ApiClient,
    _req: PageRequest,
) -> Result<DjProgramPage, Failure> {
    let q = Query::new();
    let res = client.dj_recommend(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["djRadios"]
        .as_array()
        .map(|arr| arr.iter().map(parse_dj_program).collect())
        .unwrap_or_default();

    Ok(DjProgramPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_dj_program(client: &ApiClient, req: IdRequest) -> Result<DjProgram, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.dj_program_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["program"].is_object() {
        Ok(parse_dj_program(&res.body["program"]))
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("DJ Program {} not found", req.id),
        })
    }
}

pub async fn get_user_dj_radios(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<DjRadioPage, Failure> {
    get_subscribed_dj_radios(
        client,
        PageRequest {
            limit: req.limit,
            offset: req.offset,
        },
    )
    .await
}

pub async fn search_dj_radios(
    client: &ApiClient,
    req: SearchRequest,
) -> Result<DjRadioPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "1009".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["result"]["djRadios"]
        .as_array()
        .map(|arr| arr.iter().map(parse_dj_radio).collect())
        .unwrap_or_default();

    let count = res.body["result"]["djRadiosCount"].as_i64();

    Ok(DjRadioPage {
        items,
        total: count,
        hasMore: None,
    })
}

pub async fn get_dj_categories(client: &ApiClient, _req: Bytes) -> Result<CategoryPage, Failure> {
    let q = Query::new();
    let res = client.dj_catelist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["categories"].as_array() {
        for c in arr {
            items.push(Category {
                id: c["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                name: c["name"].as_str().unwrap_or_default().to_string(),
            });
        }
    }

    Ok(CategoryPage { items })
}

pub async fn get_dj_radios_by_category(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<DjRadioPage, Failure> {
    let mut q = Query::new();
    q.params.insert("type".to_string(), req.id);

    let res = client.dj_recommend_type(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["djRadios"]
        .as_array()
        .map(|arr| arr.iter().map(parse_dj_radio).collect())
        .unwrap_or_default();

    Ok(DjRadioPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_dj_radio_subscribers(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<SocialUserPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.dj_subscriber(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["subscribers"]
        .as_array()
        .map(|arr| arr.iter().map(parse_social_user).collect())
        .unwrap_or_default();

    let more = res.body["hasMore"].as_bool();

    Ok(SocialUserPage {
        items,
        total: None,
        hasMore: more,
    })
}

pub async fn browse_dj_ranking(
    client: &ApiClient,
    req: DjRankingRequest,
) -> Result<DjRadioPage, Failure> {
    let mut q = Query::new();
    q.params.insert("type".to_string(), req.kind);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.dj_radio_top(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["toplist"]
        .as_array()
        .map(|arr| arr.iter().map(parse_dj_radio).collect())
        .unwrap_or_default();

    Ok(DjRadioPage {
        items,
        total: None,
        hasMore: None,
    })
}
