//! User profiles, listening records, library likes, following, cloud library, and authentication.

use super::parsers::{parse_album, parse_catalog_artist, parse_dj_radio, parse_playlist, parse_track, parse_user_profile, parse_video_item};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn get_user_profile(client: &ApiClient, req: UserRequest) -> Result<UserProfile, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id.clone());

    let res = client.user_detail(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["profile"].is_object() {
        Ok(parse_user_profile(&res.body["profile"]))
    } else {
        Err(Failure {
            category: "NotFound".to_string(),
            message: format!("User {} not found", req.id),
        })
    }
}

pub async fn get_user_records(client: &ApiClient, req: UserRecordsRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("type".to_string(), req.r#type.to_string());

    let res = client.user_record(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let record_list = res.body["allData"]
        .as_array()
        .or_else(|| res.body["weekData"].as_array());

    let items = match record_list {
        Some(arr) => arr.iter().map(|item| parse_track(&item["song"])).collect(),
        None => vec![],
    };

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_user_follows(client: &ApiClient, req: UserListRequest) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.user_follows(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["follow"]
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

pub async fn get_user_followeds(client: &ApiClient, req: UserListRequest) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.user_followeds(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["followeds"]
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

pub async fn get_user_cloud(client: &ApiClient, req: UserListRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.user_cloud(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(|item| parse_track(&item["simpleSong"])).collect())
        .unwrap_or_default();

    let has_more = res.body["hasMore"].as_bool();
    let count = res.body["count"].as_i64();

    Ok(TrackPage {
        items,
        total: count,
        hasMore: has_more,
    })
}

pub async fn get_like_list(client: &ApiClient, req: UserRequest) -> Result<IdList, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);

    let res = client.likelist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["ids"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|n| n.to_string())).collect())
        .unwrap_or_default();

    Ok(IdList { items })
}

pub async fn get_like_list_page(client: &ApiClient, req: PagedIdRequest) -> Result<IdPage, Failure> {
    let all = get_like_list(client, UserRequest { id: req.id }).await?;
    let total = all.items.len();
    let offset = req.offset.max(0) as usize;
    let limit = if req.limit <= 0 { 30 } else { req.limit as usize };
    let items = if offset >= total {
        vec![]
    } else {
        let end = (offset + limit).min(total);
        all.items[offset..end].to_vec()
    };
    let has_more = offset + items.len() < total;
    Ok(IdPage {
        items,
        total: Some(total as i64),
        hasMore: has_more,
    })
}

pub async fn get_user_albums(client: &ApiClient, req: UserListRequest) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.album_sublist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| arr.iter().map(parse_album).collect())
        .unwrap_or_default();

    let has_more = res.body["hasMore"].as_bool();
    let count = res.body["count"].as_i64();

    Ok(AlbumPage {
        items,
        total: count,
        hasMore: has_more,
    })
}

pub async fn get_current_account(client: &ApiClient, _req: Bytes) -> Result<AccountSession, Failure> {
    let q = Query::new();
    let res = client.user_account(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    if res.body["profile"].is_object() {
        let p = &res.body["profile"];
        Ok(AccountSession {
            loggedIn: true,
            userId: p["userId"].as_i64().map(|n| n.to_string()),
            nickname: p["nickname"].as_str().map(String::from),
            avatarUrl: p["avatarUrl"].as_str().map(String::from),
        })
    } else {
        Ok(AccountSession {
            loggedIn: false,
            userId: None,
            nickname: None,
            avatarUrl: None,
        })
    }
}

pub async fn authenticate(client: &ApiClient, req: AuthActionRequest) -> Result<AuthActionResult, Failure> {
    match req.action.as_str() {
        "qr_key" => {
            let q = Query::new();
            let res = client.login_qr_key(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let key = res.body["data"]["unkey"].as_str().map(String::from);
            Ok(AuthActionResult {
                success: key.is_some(),
                key,
                ..Default::default()
            })
        }
        "qr_create" => {
            let mut q = Query::new();
            if let Some(k) = req.key {
                q.params.insert("key".to_string(), k);
            }
            q.params.insert("qrimg".to_string(), "true".to_string());
            let res = client.login_qr_create(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let img = res.body["data"]["qrimg"].as_str().map(String::from);
            let url = res.body["data"]["qrurl"].as_str().map(String::from);
            Ok(AuthActionResult {
                success: img.is_some(),
                imageBase64: img,
                url,
                ..Default::default()
            })
        }
        "qr_check" => {
            let mut q = Query::new();
            if let Some(k) = req.key {
                q.params.insert("key".to_string(), k);
            }
            let res = client.login_qr_check(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let code = res.body["code"].as_i64();
            let cookie = res.body["cookie"].as_str().map(String::from);
            Ok(AuthActionResult {
                success: code == Some(803),
                code: code.map(|n| n as i32),
                cookie,
                message: res.body["message"].as_str().map(String::from),
                ..Default::default()
            })
        }
        "cellphone" => {
            let mut q = Query::new();
            if let Some(phone) = req.phone {
                q.params.insert("phone".to_string(), phone);
            }
            if let Some(pwd) = req.password {
                q.params.insert(if req.md5 { "md5_password" } else { "password" }.to_string(), pwd);
            }
            if let Some(captcha) = req.captcha {
                q.params.insert("captcha".to_string(), captcha);
            }
            let res = client.login_cellphone(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let code = res.body["code"].as_i64();
            let cookie = res.body["cookie"].as_str().map(String::from);
            Ok(AuthActionResult {
                success: code == Some(200),
                code: code.map(|n| n as i32),
                cookie,
                message: res.body["message"].as_str().map(String::from),
                ..Default::default()
            })
        }
        _ => Err(Failure {
            category: "Unsupported".to_string(),
            message: format!("Unsupported auth action: {}", req.action),
        }),
    }
}

pub async fn update_user_profile(client: &ApiClient, req: UserProfileUpdateRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    if let Some(nick) = req.nickname {
        q.params.insert("nickname".to_string(), nick);
    }
    if let Some(sig) = req.signature {
        q.params.insert("signature".to_string(), sig);
    }
    if let Some(gender) = req.gender {
        q.params.insert("gender".to_string(), gender.to_string());
    }
    if let Some(birthday) = req.birthdayEpochMs {
        q.params.insert("birthday".to_string(), birthday.to_string());
    }
    if let Some(province) = req.province {
        q.params.insert("province".to_string(), province.to_string());
    }
    if let Some(city) = req.city {
        q.params.insert("city".to_string(), city.to_string());
    }

    let res = client.user_update(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn daily_signin(client: &ApiClient, _req: Bytes) -> Result<SigninResult, Failure> {
    let q = Query::new();
    let res = client.daily_signin(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    let point = res.body["point"].as_i64().map(|n| n as i32);
    Ok(SigninResult {
        success: code == 200,
        points: point,
    })
}

pub async fn follow_user(client: &ApiClient, req: FollowUserRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("t".to_string(), if req.followed { "1" } else { "0" }.to_string());

    let res = client.follow(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_recent_tracks(client: &ApiClient, req: PageRequest) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.record_recent_song(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]["list"]
        .as_array()
        .map(|arr| arr.iter().map(|item| parse_track(&item["data"])).collect())
        .unwrap_or_default();

    Ok(TrackPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_recent_albums(client: &ApiClient, req: PageRequest) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.record_recent_album(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]["list"]
        .as_array()
        .map(|arr| arr.iter().map(|item| parse_album(&item["data"])).collect())
        .unwrap_or_default();

    Ok(AlbumPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_recent_playlists(client: &ApiClient, req: PageRequest) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.record_recent_playlist(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]["list"]
        .as_array()
        .map(|arr| arr.iter().map(|item| parse_playlist(&item["data"])).collect())
        .unwrap_or_default();

    Ok(PlaylistPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_recent_videos(client: &ApiClient, req: PageRequest) -> Result<VideoPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.record_recent_video(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]["list"]
        .as_array()
        .map(|arr| arr.iter().map(|item| parse_video_item(&item["data"])).collect())
        .unwrap_or_default();

    Ok(VideoPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_recent_dj_radios(client: &ApiClient, req: PageRequest) -> Result<DjRadioPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.record_recent_dj(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]["list"]
        .as_array()
        .map(|arr| arr.iter().map(|item| parse_dj_radio(&item["data"])).collect())
        .unwrap_or_default();

    Ok(DjRadioPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn cloud_import(client: &ApiClient, req: CloudImportRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("songid".to_string(), req.id.clone());
    q.params.insert("adjustid".to_string(), req.matchId);

    let res = client.cloud_match(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn cloud_match(client: &ApiClient, req: CloudMatchRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("userId".to_string(), req.userId);
    q.params.insert("songId".to_string(), req.id.clone());
    q.params.insert("adjustId".to_string(), req.adjustId);

    let res = client.cloud_match(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn delete_user_cloud(client: &ApiClient, req: BatchRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    for (i, id) in req.ids.iter().enumerate() {
        q.params.insert(format!("songid[{}]", i), id.clone());
    }

    let res = client.user_cloud_del(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn get_user_bindings(client: &ApiClient, req: UserRequest) -> Result<AccountBindingPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);

    let res = client.user_binding(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["bindings"].as_array() {
        for b in arr {
            items.push(AccountBinding {
                id: b["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                r#type: b["type"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                name: None,
                maskedAccount: None,
            });
        }
    }

    Ok(AccountBindingPage { items })
}

pub async fn bind_phone(client: &ApiClient, req: PhoneBindingRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("cellphone".to_string(), req.phone);
    q.params.insert("countrycode".to_string(), req.countryCode);
    q.params.insert("captcha".to_string(), req.captcha);

    let res = client.cellphone_existence_check(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn replace_phone(client: &ApiClient, req: PhoneReplacementRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("phone".to_string(), req.phone);
    q.params.insert("countrycode".to_string(), req.countryCode);
    q.params.insert("captcha".to_string(), req.captcha);
    q.params.insert("oldcaptcha".to_string(), req.oldCaptcha);

    let res = client.user_replacephone(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn like_song(client: &ApiClient, req: SongLikeRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("like".to_string(), req.liked.to_string());

    let res = client.like(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn dislike_song(client: &ApiClient, req: IdRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.fm_trash(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}
