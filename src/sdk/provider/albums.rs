//! Album browsing, details, subscriptions, digital albums, and charts.

use super::parsers::{parse_album, parse_track};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn search_albums(client: &ApiClient, req: SearchRequest) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "10".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let result = &res.body["result"];
    let albums = result["albums"].as_array();
    let total = result["albumCount"].as_i64();

    let items = match albums {
        Some(arr) => arr.iter().map(parse_album).collect(),
        None => vec![],
    };

    let has_more = match total {
        Some(t) => Some((req.offset as i64 + items.len() as i64) < t),
        None => None,
    };

    Ok(AlbumPage {
        items,
        total,
        hasMore: has_more,
    })
}

pub async fn get_album(client: &ApiClient, req: IdRequest) -> Result<AlbumDetails, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.album(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["album"].is_object() {
        let album = parse_album(&res.body["album"]);
        let tracks = res.body["songs"]
            .as_array()
            .map(|arr| arr.iter().map(parse_track).collect())
            .unwrap_or_default();
        Ok(AlbumDetails { album, tracks })
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("Album {} not found", req.id),
        })
    }
}

pub async fn subscribe_album(client: &ApiClient, req: AlbumSubscribeRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("t".to_string(), if req.subscribed { "1" } else { "0" }.to_string());

    let res = client.album_sub(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_digital_albums(client: &ApiClient, req: PageRequest) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.digital_album_purchased(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["paidAlbums"]
        .as_array()
        .map(|arr| arr.iter().map(parse_album).collect())
        .unwrap_or_default();

    Ok(AlbumPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn browse_new_albums(client: &ApiClient, req: AlbumPageRequest) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("area".to_string(), req.area);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.album_new(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["albums"]
        .as_array()
        .map(|arr| arr.iter().map(parse_album).collect())
        .unwrap_or_default();

    let total = res.body["total"].as_i64();

    Ok(AlbumPage {
        items,
        total,
        hasMore: None,
    })
}

pub async fn get_latest_albums(client: &ApiClient, _req: Bytes) -> Result<AlbumPage, Failure> {
    let q = Query::new();
    let res = client.album_newest(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["albums"]
        .as_array()
        .map(|arr| arr.iter().map(parse_album).collect())
        .unwrap_or_default();

    Ok(AlbumPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_album_chart(client: &ApiClient, req: AlbumChartRequest) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("albumType".to_string(), if req.kind == "new" { "0" } else { "1" }.to_string());
    q.params.insert("type".to_string(), req.area);

    let res = client.album_songsaleboard(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["products"]
        .as_array()
        .map(|arr| arr.iter().map(parse_album).collect())
        .unwrap_or_default();

    Ok(AlbumPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_new_albums(client: &ApiClient, req: AlbumBrowseRequest) -> Result<AlbumPage, Failure> {
    browse_new_albums(client, AlbumPageRequest {
        area: req.area,
        limit: req.limit,
        offset: 0,
    }).await
}
