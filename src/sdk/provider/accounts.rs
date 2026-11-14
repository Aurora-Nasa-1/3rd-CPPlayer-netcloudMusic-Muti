//! User profiles, listening records, library likes, following, cloud library, and authentication.

use super::parsers::{
    parse_album, parse_catalog_artist, parse_dj_radio, parse_playlist, parse_track,
    parse_user_profile, parse_video_item,
};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn get_user_profile(
    client: &ApiClient,
    req: UserRequest,
) -> Result<UserProfile, Failure> {
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

pub async fn get_user_records(
    client: &ApiClient,
    req: UserRecordsRequest,
) -> Result<TrackPage, Failure> {
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

pub async fn get_user_follows(
    client: &ApiClient,
    req: UserListRequest,
) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

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

pub async fn get_user_followeds(
    client: &ApiClient,
    req: UserListRequest,
) -> Result<ArtistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

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

pub async fn get_user_cloud(
    client: &ApiClient,
    req: UserListRequest,
) -> Result<TrackPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.user_cloud(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_track(&item["simpleSong"]))
                .collect()
        })
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
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|n| n.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(IdList { items })
}

pub async fn get_like_list_page(
    client: &ApiClient,
    req: PagedIdRequest,
) -> Result<IdPage, Failure> {
    let all = get_like_list(client, UserRequest { id: req.id }).await?;
    let total = all.items.len();
    let offset = req.offset.max(0) as usize;
    let limit = if req.limit <= 0 {
        30
    } else {
        req.limit as usize
    };
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

pub async fn get_user_albums(
    client: &ApiClient,
    req: UserListRequest,
) -> Result<AlbumPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

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

pub async fn get_current_account(
    client: &ApiClient,
    _req: Bytes,
) -> Result<AccountSession, Failure> {
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

/// Validate the typed country-code field before mapping it to an upstream query.
fn auth_country_code(req: &AuthActionRequest) -> Result<Option<String>, Failure> {
    let Some(raw) = req.countryCode.as_deref() else {
        return Ok(None);
    };
    if !(1..=4).contains(&raw.len()) || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Failure {
            category: "Runtime".to_string(),
            message: "phone countryCode must contain 1..4 digits".to_string(),
        });
    }
    Ok(Some(raw.to_string()))
}

fn add_auth_country_code(
    query: &mut Query,
    req: &AuthActionRequest,
    upstream_key: &str,
) -> Result<(), Failure> {
    if let Some(country_code) = auth_country_code(req)? {
        query.params.insert(upstream_key.to_string(), country_code);
    }
    Ok(())
}

/// 读取登录风控链接：上游不同接口分别放在 `verifyUrl` / `redirectUrl`。
fn auth_verify_url(body: &serde_json::Value) -> Option<String> {
    body["verifyUrl"]
        .as_str()
        .or_else(|| body["data"]["verifyUrl"].as_str())
        .or_else(|| body["redirectUrl"].as_str())
        .or_else(|| body["data"]["redirectUrl"].as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// 账号风控的判定：`250` 是验证码挑战，`10004` 是邮箱/手机号的「账号有安全风险」。
fn auth_challenge(body: &serde_json::Value, code: Option<i64>) -> bool {
    matches!(code, Some(250) | Some(10004))
        || body["needVerify"].as_bool().unwrap_or(false)
        || auth_verify_url(body).is_some()
}

fn auth_result_from_body(
    body: &serde_json::Value,
    response_cookies: &[String],
) -> AuthActionResult {
    let code = body["code"].as_i64().or_else(|| body["status"].as_i64());
    let cookie = body["cookie"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(String::from)
        .or_else(|| {
            if response_cookies.is_empty() {
                None
            } else {
                Some(response_cookies.join("; "))
            }
        });
    let challenge = auth_challenge(body, code);
    AuthActionResult {
        success: code == Some(200) && !challenge,
        code: code.map(|n| n as i32),
        status: code.map(|n| n as i32),
        cookie,
        challenge,
        verifyUrl: auth_verify_url(body),
        message: body["message"]
            .as_str()
            .or_else(|| body["msg"].as_str())
            .map(String::from),
        ..Default::default()
    }
}

fn phone_login_result(body: &serde_json::Value, response_cookies: &[String]) -> AuthActionResult {
    auth_result_from_body(body, response_cookies)
}

pub async fn authenticate(
    client: &ApiClient,
    req: AuthActionRequest,
) -> Result<AuthActionResult, Failure> {
    match req.action.to_ascii_lowercase().as_str() {
        "qr.key" | "qr_key" | "qrkey" => {
            let q = Query::new();
            let res = client.login_qr_key(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let key = res.body["data"]["unkey"]
                .as_str()
                .or_else(|| res.body["unikey"].as_str())
                .or_else(|| res.body["unkey"].as_str())
                .or_else(|| res.body["data"].as_str())
                .map(String::from);
            let code = res.body["code"]
                .as_i64()
                .or_else(|| res.body["data"]["code"].as_i64())
                .map(|n| n as i32);
            let success = key.is_some();
            Ok(AuthActionResult {
                success,
                code,
                status: code,
                key,
                message: res.body["message"].as_str().map(String::from),
                ..Default::default()
            })
        }
        "qr.create" | "qr_create" | "qrcreate" => {
            let mut q = Query::new();
            if let Some(k) = &req.key {
                q.params.insert("key".to_string(), k.clone());
            }
            q.params.insert("qrimg".to_string(), "true".to_string());
            let res = client.login_qr_create(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let url = res.body["data"]["qrurl"]
                .as_str()
                .map(String::from)
                .or_else(|| {
                    req.key
                        .as_ref()
                        .map(|k| format!("https://music.163.com/login?codekey={}", k))
                });
            let img = res.body["data"]["qrimg"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(String::from);
            let success = url.is_some() || img.is_some();
            Ok(AuthActionResult {
                success,
                code: Some(200),
                status: Some(200),
                url,
                imageBase64: img,
                ..Default::default()
            })
        }
        "qr.check" | "qr_check" | "qrcheck" => {
            let mut q = Query::new();
            if let Some(k) = req.key {
                q.params.insert("key".to_string(), k);
            }
            let res = client.login_qr_check(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let code = res.body["code"]
                .as_i64()
                .or_else(|| res.body["status"].as_i64());
            let cookie = res.body["cookie"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .or_else(|| {
                    if !res.cookie.is_empty() {
                        Some(res.cookie.join("; "))
                    } else {
                        None
                    }
                });
            let success = code == Some(803);
            Ok(AuthActionResult {
                success,
                code: code.map(|n| n as i32),
                status: code.map(|n| n as i32),
                cookie,
                message: res.body["message"].as_str().map(String::from),
                ..Default::default()
            })
        }
        "login.phone" | "login_phone" | "cellphone" | "phone" => {
            let mut q = Query::new();
            add_auth_country_code(&mut q, &req, "countrycode")?;
            if let Some(phone) = req.phone {
                q.params.insert("phone".to_string(), phone);
            }
            if let Some(pwd) = req.password {
                q.params.insert(
                    if req.md5 { "md5_password" } else { "password" }.to_string(),
                    pwd,
                );
            }
            if let Some(captcha) = req.captcha {
                q.params.insert("captcha".to_string(), captcha);
            }
            let res = client.login_cellphone(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            Ok(phone_login_result(&res.body, &res.cookie))
        }
        "login.email" | "login_email" | "email" => {
            let mut q = Query::new();
            if let Some(email) = req.email {
                q.params.insert("email".to_string(), email);
            }
            if let Some(pwd) = req.password {
                q.params.insert(
                    if req.md5 { "md5_password" } else { "password" }.to_string(),
                    pwd,
                );
            }
            let res = client.login(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            Ok(auth_result_from_body(&res.body, &res.cookie))
        }
        "captcha.send" | "captcha_send" | "captcha_sent" => {
            let mut q = Query::new();
            add_auth_country_code(&mut q, &req, "ctcode")?;
            if let Some(phone) = req.phone {
                q.params.insert("phone".to_string(), phone);
            }
            let res = client.captcha_sent(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            Ok(auth_result_from_body(&res.body, &res.cookie))
        }
        "login.anonymous" | "login_anonymous" | "anonymous" => {
            let q = Query::new();
            let res = client.register_anonimous(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let code = res.body["code"].as_i64();
            let cookie = res.body["cookie"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .or_else(|| {
                    if !res.cookie.is_empty() {
                        Some(res.cookie.join("; "))
                    } else {
                        None
                    }
                });
            Ok(AuthActionResult {
                success: code == Some(200) || cookie.is_some(),
                code: code.map(|n| n as i32),
                status: code.map(|n| n as i32),
                cookie,
                message: res.body["message"]
                    .as_str()
                    .or_else(|| res.body["msg"].as_str())
                    .map(String::from),
                ..Default::default()
            })
        }
        "logout" => {
            let q = Query::new();
            let res = client.logout(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let code = res.body["code"].as_i64();
            Ok(AuthActionResult {
                success: code == Some(200) || code.is_none(),
                code: code.map(|n| n as i32),
                status: code.map(|n| n as i32),
                ..Default::default()
            })
        }
        "verify.url" | "verify_url" => {
            let mut q = Query::new();
            if let Some(vid) = req.verifyId {
                q.params.insert("verifyId".to_string(), vid);
            }
            let res = client.verify_get_qr(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let code = res.body["code"].as_i64();
            let qr_code = res.body["data"]["qrCode"].as_str().map(String::from);
            let qr_url = res.body["data"]["qrurl"].as_str().map(String::from);
            Ok(AuthActionResult {
                success: code == Some(200) || qr_url.is_some(),
                code: code.map(|n| n as i32),
                status: code.map(|n| n as i32),
                key: qr_code,
                url: qr_url,
                message: res.body["message"].as_str().map(String::from),
                ..Default::default()
            })
        }
        "verify.check" | "verify_check" => {
            let mut q = Query::new();
            if let Some(k) = req.key {
                q.params.insert("qr".to_string(), k);
            }
            let res = client.verify_qrcodestatus(&q).await.map_err(|e| Failure {
                category: "Authentication".to_string(),
                message: e.to_string(),
            })?;
            let code = res.body["code"].as_i64();
            let cookie = res.body["cookie"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .or_else(|| {
                    if !res.cookie.is_empty() {
                        Some(res.cookie.join("; "))
                    } else {
                        None
                    }
                });
            Ok(AuthActionResult {
                success: code == Some(200) || code == Some(803),
                code: code.map(|n| n as i32),
                status: code.map(|n| n as i32),
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

pub async fn update_user_profile(
    client: &ApiClient,
    req: UserProfileUpdateRequest,
) -> Result<MutationResult, Failure> {
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
        q.params
            .insert("birthday".to_string(), birthday.to_string());
    }
    if let Some(province) = req.province {
        q.params
            .insert("province".to_string(), province.to_string());
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

pub async fn follow_user(
    client: &ApiClient,
    req: FollowUserRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert(
        "t".to_string(),
        if req.followed { "1" } else { "0" }.to_string(),
    );

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

pub async fn get_recent_playlists(
    client: &ApiClient,
    req: PageRequest,
) -> Result<PlaylistPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client
        .record_recent_playlist(&q)
        .await
        .map_err(|e| Failure {
            category: "Network".to_string(),
            message: e.to_string(),
        })?;

    let items = res.body["data"]["list"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_playlist(&item["data"]))
                .collect()
        })
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
        .map(|arr| {
            arr.iter()
                .map(|item| parse_video_item(&item["data"]))
                .collect()
        })
        .unwrap_or_default();

    Ok(VideoPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_recent_dj_radios(
    client: &ApiClient,
    req: PageRequest,
) -> Result<DjRadioPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.record_recent_dj(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let items = res.body["data"]["list"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| parse_dj_radio(&item["data"]))
                .collect()
        })
        .unwrap_or_default();

    Ok(DjRadioPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn cloud_import(
    client: &ApiClient,
    req: CloudImportRequest,
) -> Result<MutationResult, Failure> {
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

pub async fn cloud_match(
    client: &ApiClient,
    req: CloudMatchRequest,
) -> Result<MutationResult, Failure> {
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

pub async fn delete_user_cloud(
    client: &ApiClient,
    req: BatchRequest,
) -> Result<MutationResult, Failure> {
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

pub async fn get_user_bindings(
    client: &ApiClient,
    req: UserRequest,
) -> Result<AccountBindingPage, Failure> {
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
                r#type: b["type"]
                    .as_i64()
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
                name: None,
                maskedAccount: None,
            });
        }
    }

    Ok(AccountBindingPage { items })
}

pub async fn bind_phone(
    client: &ApiClient,
    req: PhoneBindingRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("cellphone".to_string(), req.phone);
    q.params.insert("countrycode".to_string(), req.countryCode);
    q.params.insert("captcha".to_string(), req.captcha);

    let res = client
        .cellphone_existence_check(&q)
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

pub async fn replace_phone(
    client: &ApiClient,
    req: PhoneReplacementRequest,
) -> Result<MutationResult, Failure> {
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

pub async fn like_song(
    client: &ApiClient,
    req: SongLikeRequest,
) -> Result<MutationResult, Failure> {
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

#[cfg(test)]
mod phone_auth_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn typed_country_code_validates_and_maps_to_upstream_parameter_names() {
        let req = AuthActionRequest {
            action: "login.phone".to_string(),
            countryCode: Some("44".to_string()),
            ..Default::default()
        };
        assert_eq!(auth_country_code(&req).unwrap().as_deref(), Some("44"));

        let four_digits = AuthActionRequest {
            countryCode: Some("1234".to_string()),
            ..Default::default()
        };
        assert_eq!(
            auth_country_code(&four_digits).unwrap().as_deref(),
            Some("1234")
        );

        for invalid_code in ["", "+44", "12345", "4x"] {
            let invalid = AuthActionRequest {
                countryCode: Some(invalid_code.to_string()),
                ..Default::default()
            };
            assert_eq!(
                auth_country_code(&invalid).unwrap_err().category,
                "Runtime",
                "invalid country code: {invalid_code}"
            );
        }

        let mut login = Query::new();
        add_auth_country_code(&mut login, &req, "countrycode").unwrap();
        assert_eq!(
            login.params.get("countrycode").map(String::as_str),
            Some("44")
        );
        let mut captcha = Query::new();
        add_auth_country_code(&mut captcha, &req, "ctcode").unwrap();
        assert_eq!(captcha.params.get("ctcode").map(String::as_str), Some("44"));

        let omitted = AuthActionRequest::default();
        assert_eq!(auth_country_code(&omitted).unwrap(), None);
        let mut default_query = Query::new();
        add_auth_country_code(&mut default_query, &omitted, "countrycode").unwrap();
        assert!(!default_query.params.contains_key("countrycode"));
    }

    #[test]
    fn phone_login_result_returns_success_and_cookie_from_response_headers() {
        let result = phone_login_result(
            &json!({ "code": 200, "msg": "ok" }),
            &["MUSIC_U=token".to_string(), "NMTID=abc".to_string()],
        );
        assert!(result.success);
        assert_eq!(result.code, Some(200));
        assert_eq!(result.status, Some(200));
        assert_eq!(result.cookie.as_deref(), Some("MUSIC_U=token; NMTID=abc"));
        assert_eq!(result.message.as_deref(), Some("ok"));
    }

    #[test]
    fn phone_login_result_preserves_challenge_without_claiming_success() {
        let result = phone_login_result(
            &json!({ "code": 250, "data": { "verifyUrl": "https://example.test/verify" } }),
            &[],
        );
        assert!(!result.success);
        assert!(result.challenge);
        assert_eq!(result.code, Some(250));
        assert_eq!(
            result.verifyUrl.as_deref(),
            Some("https://example.test/verify")
        );
        assert!(result.cookie.is_none());

        let flag_only = phone_login_result(&json!({ "code": 200, "needVerify": true }), &[]);
        assert!(flag_only.challenge);
        assert!(!flag_only.success);
    }

    #[test]
    fn phone_login_result_does_not_treat_failure_as_success_and_body_cookie_wins() {
        let result = phone_login_result(
            &json!({ "status": 401, "cookie": "body-cookie", "message": "denied" }),
            &["header-cookie".to_string()],
        );
        assert!(!result.success);
        assert!(!result.challenge);
        assert_eq!(result.code, Some(401));
        assert_eq!(result.cookie.as_deref(), Some("body-cookie"));
        assert_eq!(result.message.as_deref(), Some("denied"));
    }

    #[test]
    fn email_login_result_maps_redirect_url_to_verify_url() {
        let result = auth_result_from_body(
            &json!({
                "code": 10004,
                "loginType": 0,
                "message": "账号有安全风险",
                "redirectUrl": "https://id.163.com/mail/stable.html?username=a%40b.c"
            }),
            &[],
        );
        assert!(!result.success);
        assert!(result.challenge);
        assert_eq!(result.code, Some(10004));
        assert_eq!(
            result.verifyUrl.as_deref(),
            Some("https://id.163.com/mail/stable.html?username=a%40b.c")
        );
        assert!(result.cookie.is_none());
    }

    #[test]
    fn email_login_result_reads_nested_redirect_url_and_reports_password_error() {
        let nested = auth_result_from_body(
            &json!({ "code": 250, "data": { "redirectUrl": "https://example.test/verify" } }),
            &[],
        );
        assert!(nested.challenge);
        assert!(!nested.success);
        assert_eq!(
            nested.verifyUrl.as_deref(),
            Some("https://example.test/verify")
        );

        let bad_password = auth_result_from_body(
            &json!({ "code": 502, "msg": "账号或密码错误", "message": "账号或密码错误" }),
            &[],
        );
        assert!(!bad_password.success);
        assert!(!bad_password.challenge);
        assert_eq!(bad_password.message.as_deref(), Some("账号或密码错误"));
    }
}
