//! Artist searching, details, songs, albums, subscriptions, and browsing.

use super::parsers::{ncm_err, parse_album, parse_catalog_artist, parse_track};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn search_artists(client: &ApiClient, req: SearchRequest) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "100".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let result = &res.body["result"];
    let artists = result["artists"].as_array();
    let total = result["artistCount"].as_i64();

    let items = match artists {
        Some(arr) => arr.iter().map(parse_catalog_artist).collect(),
        None => vec![],
    };

    let has_more = match total {
        Some(t) => Some((req.offset as i64 + items.len() as i64) < t),
        None => None,
    };

    Ok(ArtistPage {
        items,
        total,
        hasMore: has_more,
    })
}

fn artist_from(body: &serde_json::Value) -> Option<CatalogArtist> {
    [&body["data"]["artist"], &body["artist"]]
        .into_iter()
        .filter(|item| item.is_object())
        .map(parse_catalog_artist)
        .find(|artist| !artist.id.is_empty() && !artist.name.is_empty())
}

fn invalid_response(endpoint: &str) -> Failure {
    Failure {
        category: "InvalidResponse".into(),
        message: format!("Malformed response from {endpoint}"),
    }
}

pub async fn get_artist(client: &ApiClient, req: IdRequest) -> Result<CatalogArtist, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    match client.artist_detail(&q).await.map_err(ncm_err) {
        Ok(res) => {
            if let Some(artist) = artist_from(&res.body) {
                return Ok(artist);
            }
        }
        Err(error) if matches!(error.category.as_str(), "NotFound" | "Runtime" | "InvalidResponse") => {}
        Err(error) => return Err(error),
    }
    let fallback = client.artists(&q).await.map_err(ncm_err)?;
    artist_from(&fallback.body).ok_or_else(|| invalid_response("artists"))
}

pub async fn get_artist_tracks(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let failure = match client.artist_songs(&q).await.map_err(ncm_err) {
        Ok(res) => {
            let data = if res.body["data"]["songs"].is_array() { &res.body["data"] } else { &res.body };
            if let Some(songs) = data["songs"].as_array() {
                return Ok(TrackPage {
                    items: songs.iter().map(parse_track).collect(),
                    total: data["total"].as_i64().or_else(|| res.body["total"].as_i64()),
                    hasMore: data["more"].as_bool().or_else(|| res.body["more"].as_bool()),
                });
            }
            invalid_response("artist/songs")
        }
        Err(error) => error,
    };

    // The artist profile requests the first 50 songs in hot order. The
    // dedicated top-song endpoint can serve that view if artist/songs fails.
    if req.offset != 0 || req.limit != 50
        || !matches!(failure.category.as_str(), "Runtime" | "InvalidResponse" | "NotFound")
    {
        return Err(failure);
    }
    let fallback = client.artist_top_song(&q).await.map_err(ncm_err)?;
    let songs = fallback.body["songs"].as_array().ok_or_else(|| invalid_response("artist/top/song"))?;
    Ok(TrackPage {
        items: songs.iter().take(50).map(parse_track).collect(),
        total: None,
        hasMore: Some(false),
    })
}

pub async fn get_artist_albums(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.artist_album(&q).await.map_err(ncm_err)?;
    let albums = res.body["hotAlbums"].as_array().ok_or_else(|| invalid_response("artist/album"))?;
    let more = res.body["more"].as_bool();

    let items = albums.iter().map(parse_album).collect();

    Ok(AlbumPage {
        items,
        total: None,
        hasMore: more,
    })
}

pub async fn get_hot_artists(
    client: &ApiClient,
    req: RecommendationsRequest,
) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.top_artists(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let artists = res.body["artists"].as_array();
    let items = match artists {
        Some(arr) => arr.iter().map(parse_catalog_artist).collect(),
        None => vec![],
    };

    Ok(ArtistPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_similar_artists(
    client: &ApiClient,
    req: IdRequest,
) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);

    let res = client.simi_artist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["artists"]
        .as_array()
        .map(|arr| arr.iter().map(parse_catalog_artist).collect())
        .unwrap_or_default();

    Ok(ArtistPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn subscribe_artist(
    client: &ApiClient,
    req: SubscriptionRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert(
        "t".to_string(),
        if req.subscribed { "1" } else { "0" }.to_string(),
    );

    let res = client.artist_sub(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_subscribed_artists(
    client: &ApiClient,
    req: PageRequest,
) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.artist_sublist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(parse_catalog_artist).collect())
        .unwrap_or_default();

    let count = res.body["count"].as_i64();

    Ok(ArtistPage {
        items,
        total: count,
        hasMore: None,
    })
}

pub async fn browse_artists(
    client: &ApiClient,
    req: ArtistBrowseRequest,
) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("type".to_string(), req.r#type.to_string());
    q.params.insert("area".to_string(), req.area.to_string());
    q.params.insert("initial".to_string(), req.initial);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.artist_list(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["artists"]
        .as_array()
        .map(|arr| arr.iter().map(parse_catalog_artist).collect())
        .unwrap_or_default();

    let more = res.body["more"].as_bool();

    Ok(ArtistPage {
        items,
        total: None,
        hasMore: more,
    })
}

pub async fn get_artist_follow_count(
    client: &ApiClient,
    req: IdRequest,
) -> Result<FollowCount, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);

    let res = client.artist_follow_count(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let count = res.body["data"]["fansCnt"].as_i64().unwrap_or(0);
    Ok(FollowCount { count })
}
