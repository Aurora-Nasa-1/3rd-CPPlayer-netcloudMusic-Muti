//! Track, Audio Resolution, Lyrics, and Audio Quality operations.

use super::parsers::parse_track;
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn search_tracks(client: &ApiClient, req: SearchRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "1".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let result = &res.body["result"];
    let songs = result["songs"].as_array();
    let total = result["songCount"].as_i64();

    let items = match songs {
        Some(arr) => arr.iter().map(parse_track).collect(),
        None => vec![],
    };

    let has_more = match total {
        Some(t) => Some((req.offset as i64 + items.len() as i64) < t),
        None => None,
    };

    Ok(TrackPage {
        items,
        total,
        hasMore: has_more,
    })
}

pub async fn get_track(client: &ApiClient, req: IdRequest) -> Result<Track, Failure> {
    let mut q = Query::new();
    q.params.insert("ids".to_string(), req.id.clone());

    let res = client.song_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if let Some(song) = res.body["songs"].as_array().and_then(|arr| arr.first()) {
        Ok(parse_track(song))
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("Track {} not found", req.id),
        })
    }
}

pub async fn get_many_tracks(client: &ApiClient, req: BatchRequest) -> Result<TrackBatch, Failure> {
    if req.ids.is_empty() {
        return Ok(TrackBatch { items: vec![] });
    }

    let mut q = Query::new();
    q.params.insert("ids".to_string(), req.ids.join(","));

    let res = client.song_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut song_map = std::collections::HashMap::new();
    if let Some(arr) = res.body["songs"].as_array() {
        for s in arr {
            let t = parse_track(s);
            song_map.insert(t.id.clone(), t);
        }
    }

    let mut items = Vec::with_capacity(req.ids.len());
    for id in req.ids {
        if let Some(track) = song_map.get(&id) {
            items.push(TrackOutcome {
                id,
                track: Some(track.clone()),
                failure: None,
            });
        } else {
            items.push(TrackOutcome {
                id: id.clone(),
                track: None,
                failure: Some(Failure {
                    category: "NotFound".to_string(),
                    message: format!("Track {} not found", id),
                }),
            });
        }
    }

    Ok(TrackBatch { items })
}

pub async fn resolve_track(
    client: &ApiClient,
    req: ResolveRequest,
) -> Result<MediaResource, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("level".to_string(), req.quality.clone());

    let res = client.song_url_v1(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if let Some(data) = res.body["data"].as_array().and_then(|arr| arr.first()) {
        let url = data["url"].as_str().unwrap_or("").to_string();
        if url.is_empty() || url == "null" {
            return Err(Failure {
                category: "NotFound".to_string(),
                message: format!("No playable audio resource found for track {}", req.id),
            });
        }

        let actual_quality = data["level"].as_str().unwrap_or(&req.quality).to_string();
        let format = data["encodeType"]
            .as_str()
            .or_else(|| data["type"].as_str())
            .map(String::from);
        let bitrate = data["br"].as_i64().map(|n| n as i32);
        let size_bytes = data["size"].as_i64();

        Ok(MediaResource {
            url,
            actualQuality: actual_quality,
            headers: vec![],
            expiresAtEpochMs: None,
            format,
            bitrate,
            sizeBytes: size_bytes,
        })
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("No media found for track {}", req.id),
        })
    }
}

pub async fn get_lyrics(client: &ApiClient, req: IdRequest) -> Result<Lyrics, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.lyric_new(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let lrc = res.body["lrc"]["lyric"].as_str().map(String::from);
    let tlyric = res.body["tlyric"]["lyric"].as_str().map(String::from);
    let yrc = res.body["yrc"]["lyric"].as_str().map(String::from);
    let romalrc = res.body["romalrc"]["lyric"].as_str().map(String::from);

    if lrc.is_none() && tlyric.is_none() && yrc.is_none() && romalrc.is_none() {
        return Err(Failure {
            category: "NotFound".to_string(),
            message: format!("Lyrics not found for track {}", req.id),
        });
    }

    Ok(Lyrics {
        id: req.id,
        lrc,
        ttml: None,
        yrc,
        translation: tlyric,
        romanization: romalrc,
    })
}

pub async fn check_track_available(
    client: &ApiClient,
    req: TrackAvailabilityRequest,
) -> Result<TrackAvailability, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("br".to_string(), req.bitrate.to_string());

    let res = client.check_music(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let success = res.body["success"].as_bool().unwrap_or(false);
    let message = res.body["message"].as_str().map(String::from);

    Ok(TrackAvailability {
        available: success,
        message,
    })
}

pub async fn get_media_qualities(
    client: &ApiClient,
    req: MediaQualityRequest,
) -> Result<QualityPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.song_download_url_v1(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(data) = res.body["data"].as_object() {
        for (level, label) in &[
            ("standard", "标准"),
            ("higher", "较高"),
            ("exhigh", "极高"),
            ("lossless", "无损"),
            ("hires", "Hi-Res"),
            ("sky", "沉浸环绕声"),
        ] {
            if let Some(info) = data.get(*level) {
                let size = info["size"].as_i64();
                let br = info["br"].as_i64().map(|n| n as i32);
                items.push(QualityOption {
                    id: level.to_string(),
                    label: label.to_string(),
                    bitrate: br,
                    sizeBytes: size,
                    available: size.unwrap_or(0) > 0,
                });
            }
        }
    }

    if items.is_empty() {
        // Fallback default qualities
        items = vec![
            QualityOption {
                id: "standard".to_string(),
                label: "标准".to_string(),
                bitrate: Some(128000),
                sizeBytes: None,
                available: true,
            },
            QualityOption {
                id: "exhigh".to_string(),
                label: "极高".to_string(),
                bitrate: Some(320000),
                sizeBytes: None,
                available: true,
            },
            QualityOption {
                id: "lossless".to_string(),
                label: "无损".to_string(),
                bitrate: Some(999000),
                sizeBytes: None,
                available: true,
            },
        ];
    }

    Ok(QualityPage { items })
}

pub async fn scrobble_track(
    client: &ApiClient,
    req: ScrobbleRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("sourceid".to_string(), req.sourceId);
    q.params
        .insert("time".to_string(), req.playedSeconds.to_string());

    let res = client.scrobble(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}
