//! Artist searching, details, songs, albums, subscriptions, and browsing.

use super::parsers::{parse_album, parse_catalog_artist, parse_track};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn search_artists(client: &ApiClient, req: SearchRequest) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "100".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

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

pub async fn get_artist(client: &ApiClient, req: IdRequest) -> Result<CatalogArtist, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.artist_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["data"]["artist"].is_object() {
        Ok(parse_catalog_artist(&res.body["data"]["artist"]))
    } else if res.body["artist"].is_object() {
        Ok(parse_catalog_artist(&res.body["artist"]))
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("Artist {} not found", req.id),
        })
    }
}

pub async fn get_artist_tracks(client: &ApiClient, req: PagedIdRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.artist_songs(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let songs = res.body["songs"].as_array();
    let total = res.body["total"].as_i64();
    let more = res.body["more"].as_bool();

    let items = match songs {
        Some(arr) => arr.iter().map(parse_track).collect(),
        None => vec![],
    };

    Ok(TrackPage {
        items,
        total,
        hasMore: more,
    })
}

pub async fn get_artist_albums(client: &ApiClient, req: PagedIdRequest) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.artist_album(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let albums = res.body["hotAlbums"].as_array();
    let more = res.body["more"].as_bool();

    let items = match albums {
        Some(arr) => arr.iter().map(parse_album).collect(),
        None => vec![],
    };

    Ok(AlbumPage {
        items,
        total: None,
        hasMore: more,
    })
}

pub async fn get_hot_artists(client: &ApiClient, req: RecommendationsRequest) -> Result<ArtistPage, Failure> {
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

pub async fn get_similar_artists(client: &ApiClient, req: IdRequest) -> Result<ArtistPage, Failure> {
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

pub async fn subscribe_artist(client: &ApiClient, req: SubscriptionRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("artistId".to_string(), req.id.clone());
    q.params.insert("t".to_string(), if req.subscribed { "1" } else { "0" }.to_string());

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

pub async fn get_subscribed_artists(client: &ApiClient, req: PageRequest) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

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

pub async fn browse_artists(client: &ApiClient, req: ArtistBrowseRequest) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("type".to_string(), req.r#type.to_string());
    q.params.insert("area".to_string(), req.area.to_string());
    q.params.insert("initial".to_string(), req.initial);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

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

pub async fn get_artist_follow_count(client: &ApiClient, req: IdRequest) -> Result<FollowCount, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);

    let res = client.artist_follow_count(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let count = res.body["data"]["fansCnt"].as_i64().unwrap_or(0);
    Ok(FollowCount { count })
}
