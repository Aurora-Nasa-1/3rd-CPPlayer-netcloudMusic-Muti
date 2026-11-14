//! Playlist browsing, details, tracks, mutations, categories, and import operations.

use super::parsers::{parse_playlist, parse_social_user, parse_track};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn search_playlists(
    client: &ApiClient,
    req: SearchRequest,
) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "1000".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let result = &res.body["result"];
    let playlists = result["playlists"].as_array();
    let total = result["playlistCount"].as_i64();

    let items = match playlists {
        Some(arr) => arr.iter().map(parse_playlist).collect(),
        None => vec![],
    };

    let has_more = match total {
        Some(t) => Some((req.offset as i64 + items.len() as i64) < t),
        None => None,
    };

    Ok(PlaylistPage {
        items,
        total,
        hasMore: has_more,
    })
}

pub async fn get_playlist(client: &ApiClient, req: IdRequest) -> Result<Playlist, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.playlist_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["playlist"].is_object() {
        Ok(parse_playlist(&res.body["playlist"]))
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("Playlist {} not found", req.id),
        })
    }
}

pub async fn get_playlist_tracks(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let page = client.playlist_track_page(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let songs = page.response.body["songs"].as_array();
    let items = match songs {
        Some(arr) => arr.iter().map(parse_track).collect(),
        None => vec![],
    };

    Ok(TrackPage {
        items,
        total: Some(page.total),
        hasMore: Some(page.has_more),
    })
}

pub async fn get_user_playlists(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.user_playlist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let playlists = res.body["playlist"].as_array();
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

pub async fn delete_playlist(
    client: &ApiClient,
    req: IdRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.playlist_delete(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn subscribe_playlist(
    client: &ApiClient,
    req: PlaylistSubscribeRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.playlistId.clone());
    q.params.insert(
        "t".to_string(),
        if req.subscribed { "1" } else { "2" }.to_string(),
    );

    let res = client.playlist_subscribe(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.playlistId),
    })
}

pub async fn add_tracks_to_playlist(
    client: &ApiClient,
    req: PlaylistTracksRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("op".to_string(), "add".to_string());
    q.params.insert("pid".to_string(), req.playlistId.clone());
    q.params
        .insert("tracks".to_string(), req.trackIds.join(","));

    let res = client.playlist_tracks(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.playlistId),
    })
}

pub async fn remove_tracks_from_playlist(
    client: &ApiClient,
    req: PlaylistTracksRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("op".to_string(), "del".to_string());
    q.params.insert("pid".to_string(), req.playlistId.clone());
    q.params
        .insert("tracks".to_string(), req.trackIds.join(","));

    let res = client.playlist_tracks(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.playlistId),
    })
}

pub async fn rename_playlist(
    client: &ApiClient,
    req: PlaylistNameRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.playlistId.clone());
    q.params.insert("name".to_string(), req.name);

    let res = client.playlist_name_update(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.playlistId),
    })
}

pub async fn create_playlist(
    client: &ApiClient,
    req: CreatePlaylistRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("name".to_string(), req.name);
    q.params
        .insert("privacy".to_string(), req.privacy.to_string());

    let res = client.playlist_create(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    let id = res.body["id"].as_i64().map(|n| n.to_string());
    Ok(MutationResult {
        success: code == 200,
        id,
    })
}

pub async fn get_similar_playlists(
    client: &ApiClient,
    req: IdRequest,
) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("songid".to_string(), req.id);

    let res = client.simi_playlist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["playlists"]
        .as_array()
        .map(|arr| arr.iter().map(parse_playlist).collect())
        .unwrap_or_default();

    Ok(PlaylistPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn update_playlist(
    client: &ApiClient,
    req: PlaylistUpdateRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    if let Some(name) = req.name {
        q.params.insert("name".to_string(), name);
    }
    if let Some(desc) = req.description {
        q.params.insert("desc".to_string(), desc);
    }
    if let Some(tags) = req.tags {
        q.params.insert("tags".to_string(), tags.join(";"));
    }

    let res = client.playlist_update(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_playlist_subscribers(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<SocialUserPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.playlist_subscribers(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["subscribers"]
        .as_array()
        .map(|arr| arr.iter().map(parse_social_user).collect())
        .unwrap_or_default();

    let more = res.body["more"].as_bool();
    let total = res.body["total"].as_i64();

    Ok(SocialUserPage {
        items,
        total,
        hasMore: more,
    })
}

pub async fn get_playlist_categories(
    client: &ApiClient,
    _req: Bytes,
) -> Result<CategoryPage, Failure> {
    let q = Query::new();
    let res = client.playlist_catlist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(sub) = res.body["sub"].as_array() {
        for item in sub {
            let name = item["name"].as_str().unwrap_or_default().to_string();
            items.push(Category {
                id: name.clone(),
                name,
            });
        }
    }

    Ok(CategoryPage { items })
}

pub async fn get_hot_playlist_tags(
    client: &ApiClient,
    _req: Bytes,
) -> Result<CategoryPage, Failure> {
    let q = Query::new();
    let res = client.playlist_hot(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(tags) = res.body["tags"].as_array() {
        for item in tags {
            let name = item["name"].as_str().unwrap_or_default().to_string();
            let id = item["id"]
                .as_i64()
                .map(|n| n.to_string())
                .unwrap_or_else(|| name.clone());
            items.push(Category { id, name });
        }
    }

    Ok(CategoryPage { items })
}

pub async fn get_highquality_playlist_tags(
    client: &ApiClient,
    _req: Bytes,
) -> Result<CategoryPage, Failure> {
    let q = Query::new();
    let res = client
        .playlist_highquality_tags(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let mut items = vec![];
    if let Some(tags) = res.body["tags"].as_array() {
        for item in tags {
            let name = item["name"].as_str().unwrap_or_default().to_string();
            let id = item["id"]
                .as_i64()
                .map(|n| n.to_string())
                .unwrap_or_else(|| name.clone());
            items.push(Category { id, name });
        }
    }

    Ok(CategoryPage { items })
}

pub async fn get_created_playlists(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<PlaylistPage, Failure> {
    let res = get_user_playlists(client, req).await?;
    // Created playlists are typically those where creator.id matches user
    Ok(res)
}

pub async fn get_collected_playlists(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<PlaylistPage, Failure> {
    let res = get_user_playlists(client, req).await?;
    Ok(res)
}

pub async fn reorder_playlists(
    client: &ApiClient,
    req: PlaylistOrderRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert(
        "ids".to_string(),
        serde_json::to_string(&req.playlistIds).unwrap_or_default(),
    );

    let res = client
        .playlist_order_update(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn reorder_playlist_tracks(
    client: &ApiClient,
    req: PlaylistTrackOrderRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("pid".to_string(), req.id.clone());
    q.params.insert(
        "trackIds".to_string(),
        serde_json::to_string(&req.trackIds).unwrap_or_default(),
    );

    let res = client.song_order_update(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn update_playlist_cover(
    client: &ApiClient,
    req: PlaylistCoverRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("img_id".to_string(), req.imageId);

    let res = client
        .playlist_cover_update(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn create_playlist_import(
    client: &ApiClient,
    req: PlaylistImportRequest,
) -> Result<PlaylistImportTask, Failure> {
    let mut q = Query::new();
    let text = if let Some(t) = req.text {
        t
    } else if !req.tracks.is_empty() {
        req.tracks
            .iter()
            .map(|tr| format!("{} {}", tr.title, tr.artist.as_deref().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        req.links.join("\n")
    };
    q.params.insert("text".to_string(), text);
    q.params.insert("playlistName".to_string(), req.name);

    let res = client
        .playlist_import_name_task_create(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let task_id = res.body["data"]["taskId"]
        .as_str()
        .unwrap_or("")
        .to_string();
    Ok(PlaylistImportTask {
        taskId: task_id,
        status: "PROCESSING".to_string(),
        playlistIds: vec![],
        matchedCount: None,
        failedCount: None,
        message: None,
    })
}

pub async fn get_playlist_import_status(
    client: &ApiClient,
    req: IdRequest,
) -> Result<PlaylistImportTask, Failure> {
    let mut q = Query::new();
    q.params.insert("taskId".to_string(), req.id.clone());

    let res = client
        .playlist_import_task_status(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let status = res.body["data"]["status"]
        .as_str()
        .unwrap_or("DONE")
        .to_string();
    Ok(PlaylistImportTask {
        taskId: req.id,
        status,
        playlistIds: vec![],
        matchedCount: res.body["data"]["count"].as_i64().map(|n| n as i32),
        failedCount: None,
        message: None,
    })
}
