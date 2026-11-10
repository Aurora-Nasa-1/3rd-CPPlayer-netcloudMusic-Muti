//! Parsers for converting NetEase API JSON responses into SDK DTO models.

use crate::error::NcmError;
use crate::sdk::models::*;
use serde_json::Value;

/// Map transport/API errors onto the SDK Failure categories
/// (Unsupported / NotFound / Authentication / Network / RateLimit / Timeout /
/// InvalidResponse / Runtime). Contract rule: never collapse everything to
/// "Network" — Authentication in particular must stay distinguishable so the
/// host can prompt a login instead of retrying.
pub fn ncm_err(e: NcmError) -> Failure {
    let category = match &e {
        NcmError::Http(err) => {
            if err.is_timeout() {
                "Timeout"
            } else {
                "Network"
            }
        }
        NcmError::AuthRequired(_) => "Authentication",
        NcmError::RateLimited(_) => "RateLimit",
        NcmError::Timeout(_) => "Timeout",
        // A request the upstream rejected as malformed is a provider-side
        // defect (we built the request), not a transport problem.
        NcmError::InvalidParam(_) => "Runtime",
        NcmError::Crypto(_) | NcmError::Json(_) | NcmError::Unknown(_) => "InvalidResponse",
        NcmError::Api { code, .. } => match *code {
            301 | 881 => "Authentication",
            404 | -404 => "NotFound",
            503 => "RateLimit",
            408 => "Timeout",
            // Remaining business rejections surface the upstream message as a
            // failed operation; the request was answered, just not fulfilled.
            _ => "Runtime",
        },
    };
    Failure {
        category: category.to_string(),
        message: e.to_string(),
    }
}

/// Map the SDK resource-type names onto the numeric thread codes the
/// upstream comment endpoints expect (`RESOURCE_TYPE_MAP` keys).
/// Accepts both the contract names and the numeric codes verbatim.
pub fn comment_type_code(t: &str) -> Option<&'static str> {
    match t {
        "music" | "song" | "0" => Some("0"),
        "mv" | "1" => Some("1"),
        "playlist" | "2" => Some("2"),
        "album" | "3" => Some("3"),
        "dj" | "4" => Some("4"),
        "video" | "5" => Some("5"),
        "event" | "6" => Some("6"),
        "7" => Some("7"),
        _ => None,
    }
}

/// Map SDK sort semantics (1 = recommended, 2 = hot, 3 = latest) onto the
/// upstream `comment_new` sortType codes (99 = recommended, 2 = hot, 3 = latest).
pub fn comment_sort_upstream(sort: i32) -> i64 {
    match sort {
        2 => 2,
        3 => 3,
        _ => 99,
    }
}

/// Map an upstream `comment_new` sortType back to the SDK sort semantics.
pub fn comment_sort_sdk(sort: i64) -> i32 {
    match sort {
        2 => 2,
        3 => 3,
        99 | 1 => 1,
        other => other as i32,
    }
}

/// Extract plain message text from an upstream message field.
///
/// The upstream `msg`/`lastMsg` fields are often a *stringified* JSON envelope
/// (`{"msg":"你好","type":0}`); a few providers send plain text instead.
/// Recognise both: parse when the value looks like a JSON object and fall back
/// to the raw string. Never return the envelope itself.
pub fn extract_message_text(v: &Value) -> String {
    let raw = v.as_str().unwrap_or_default().to_string();
    if raw.trim_start().starts_with('{') {
        if let Ok(obj) = serde_json::from_str::<Value>(&raw) {
            if let Some(inner) = obj.get("msg").and_then(|m| m.as_str()) {
                return inner.to_string();
            }
        }
    }
    raw
}

/// Parse the stringified `json` envelope of a social event into
/// (plain text, image URLs, resource id, resource type).
///
/// The event list endpoints carry the payload as a JSON *string* under
/// `json`, e.g. `{"msg":"...","pics":[{"originUrl":"..."}],"song":{...}}`.
pub fn parse_event_payload(
    json_field: &Value,
) -> (String, Vec<String>, Option<String>, Option<String>) {
    let raw = json_field.as_str().unwrap_or_default();
    let payload: Value = if raw.trim_start().starts_with('{') {
        serde_json::from_str(raw).unwrap_or(Value::Null)
    } else {
        Value::Null
    };

    let text = payload
        .get("msg")
        .and_then(|m| m.as_str())
        .map(String::from)
        .unwrap_or_default();

    let image_urls = payload
        .get("pics")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|pic| {
                    if let Some(s) = pic.as_str() {
                        return Some(s.to_string());
                    }
                    pic["originUrl"]
                        .as_str()
                        .or_else(|| pic["url"].as_str())
                        .or_else(|| pic["picUrl"].as_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();

    // Associated resource: first typed payload object wins.
    let mut resource_id = None;
    let mut resource_type = None;
    for (key, kind) in [
        ("song", "song"),
        ("playlist", "playlist"),
        ("album", "album"),
        ("mv", "mv"),
        ("video", "video"),
        ("dj", "dj"),
        ("program", "dj"),
        ("event", "event"),
    ] {
        if let Some(obj) = payload.get(key).and_then(|v| v.as_object()) {
            if let Some(id) = obj
                .get("id")
                .and_then(|i| i.as_i64().map(|n| n.to_string()))
                .or_else(|| obj.get("id").and_then(|i| i.as_str().map(String::from)))
            {
                resource_id = Some(id);
                resource_type = Some(kind.to_string());
                break;
            }
        }
    }

    (text, image_urls, resource_id, resource_type)
}

/// Build a SocialEvent from one upstream event entry.
pub fn parse_event(ev: &Value) -> SocialEvent {
    let (text, image_urls, resource_id, resource_type) = parse_event_payload(&ev["json"]);
    SocialEvent {
        id: ev["id"]
            .as_i64()
            .map(|n| n.to_string())
            .or_else(|| ev["id"].as_str().map(String::from))
            .unwrap_or_default(),
        user: parse_social_user(&ev["user"]),
        text,
        publishedAtEpochMs: ev["eventTime"].as_i64().unwrap_or(0),
        resourceId: resource_id,
        resourceType: resource_type,
        imageUrls: image_urls,
    }
}

pub fn parse_artist(ar: &Value) -> Artist {
    Artist {
        id: ar["id"]
            .as_i64()
            .map(|n| n.to_string())
            .or_else(|| ar["id"].as_str().map(String::from)),
        name: ar["name"].as_str().unwrap_or_default().to_string(),
    }
}

pub fn parse_track(song: &Value) -> Track {
    let id = song["id"]
        .as_i64()
        .map(|n| n.to_string())
        .or_else(|| song["id"].as_str().map(String::from))
        .unwrap_or_default();

    let title = song["name"].as_str().unwrap_or_default().to_string();

    let artists = song["ar"]
        .as_array()
        .or_else(|| song["artists"].as_array())
        .map(|arr| arr.iter().map(parse_artist).collect())
        .unwrap_or_default();

    let duration_ms = song["dt"]
        .as_i64()
        .or_else(|| song["duration"].as_i64())
        .unwrap_or(0);

    let album = song["al"]["name"]
        .as_str()
        .or_else(|| song["album"]["name"].as_str())
        .map(String::from);

    let cover_url = song["al"]["picUrl"]
        .as_str()
        .or_else(|| song["album"]["picUrl"].as_str())
        .map(String::from);

    Track {
        id,
        title,
        artists,
        durationMs: duration_ms,
        album,
        coverUrl: cover_url,
    }
}

pub fn parse_catalog_artist(ar: &Value) -> CatalogArtist {
    let id = ar["id"]
        .as_i64()
        .map(|n| n.to_string())
        .or_else(|| ar["id"].as_str().map(String::from))
        .unwrap_or_default();

    let name = ar["name"].as_str().unwrap_or_default().to_string();

    let avatar_url = ar["picUrl"]
        .as_str()
        .or_else(|| ar["img1v1Url"].as_str())
        .map(String::from);

    let description = ar["briefDesc"]
        .as_str()
        .or_else(|| ar["desc"].as_str())
        .map(String::from);

    let aliases = ar["alias"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let album_count = ar["albumSize"].as_i64().map(|n| n as i32);
    let track_count = ar["musicSize"].as_i64().map(|n| n as i32);

    CatalogArtist {
        id,
        name,
        avatarUrl: avatar_url,
        description,
        aliases,
        albumCount: album_count,
        trackCount: track_count,
    }
}

pub fn parse_playlist(pl: &Value) -> Playlist {
    let id = pl["id"]
        .as_i64()
        .map(|n| n.to_string())
        .or_else(|| pl["id"].as_str().map(String::from))
        .unwrap_or_default();

    let title = pl["name"].as_str().unwrap_or_default().to_string();

    let cover_url = pl["coverImgUrl"]
        .as_str()
        .or_else(|| pl["picUrl"].as_str())
        .map(String::from);

    let track_count = pl["trackCount"].as_i64().map(|n| n as i32);

    let creator_id = pl["creator"]["userId"]
        .as_i64()
        .map(|n| n.to_string())
        .or_else(|| pl["creator"]["userId"].as_str().map(String::from));

    let creator_name = pl["creator"]["nickname"].as_str().map(String::from);
    let description = pl["description"].as_str().map(String::from);

    Playlist {
        id,
        title,
        coverUrl: cover_url,
        trackCount: track_count,
        creatorId: creator_id,
        creatorName: creator_name,
        description,
    }
}

pub fn parse_album(al: &Value) -> Album {
    let id = al["id"]
        .as_i64()
        .map(|n| n.to_string())
        .or_else(|| al["id"].as_str().map(String::from))
        .unwrap_or_default();

    let title = al["name"].as_str().unwrap_or_default().to_string();

    let artists = al["artists"]
        .as_array()
        .map(|arr| arr.iter().map(parse_artist).collect())
        .unwrap_or_default();

    let cover_url = al["picUrl"].as_str().map(String::from);
    let track_count = al["size"].as_i64().map(|n| n as i32);
    let released_at = al["publishTime"].as_i64();
    let company = al["company"].as_str().map(String::from);
    let description = al["description"].as_str().map(String::from);

    Album {
        id,
        title,
        artists,
        coverUrl: cover_url,
        trackCount: track_count,
        releasedAtEpochMs: released_at,
        company,
        description,
    }
}

pub fn parse_social_user(u: &Value) -> SocialUser {
    SocialUser {
        id: u["userId"]
            .as_i64()
            .map(|n| n.to_string())
            .or_else(|| u["userId"].as_str().map(String::from))
            .or_else(|| u["id"].as_i64().map(|n| n.to_string()))
            .or_else(|| u["id"].as_str().map(String::from))
            .unwrap_or_default(),
        nickname: u["nickname"]
            .as_str()
            .or_else(|| u["userName"].as_str())
            .unwrap_or_default()
            .to_string(),
        avatarUrl: u["avatarUrl"].as_str().map(String::from),
    }
}

pub fn parse_user_profile(user: &Value) -> UserProfile {
    let id = user["userId"]
        .as_i64()
        .map(|n| n.to_string())
        .unwrap_or_default();

    let nickname = user["nickname"].as_str().unwrap_or_default().to_string();
    let avatar_url = user["avatarUrl"].as_str().map(String::from);
    let signature = user["signature"].as_str().map(String::from);
    let playlist_count = user["playlistCount"].as_i64().unwrap_or(0) as i32;
    let follows = user["follows"].as_i64().unwrap_or(0) as i32;
    let followeds = user["followeds"].as_i64().unwrap_or(0) as i32;

    UserProfile {
        id,
        nickname,
        avatarUrl: avatar_url,
        signature,
        playlistCount: playlist_count,
        follows,
        followeds,
    }
}

pub fn parse_comment(c: &Value) -> CommentItem {
    let id = c["commentId"]
        .as_i64()
        .or_else(|| c["id"].as_i64())
        .unwrap_or(0);
    let content = c["content"].as_str().unwrap_or_default().to_string();
    let user = c["user"]["nickname"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let avatar = c["user"]["avatarUrl"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    // Display-time text only; never fall back to the raw numeric `time` value
    // (that would stuff "1413530574252" into a display string).
    let time = c["timeStr"].as_str().map(String::from).unwrap_or_default();
    let liked_count = c["likedCount"].as_i64().unwrap_or(0) as i32;
    let liked = c["liked"].as_bool().unwrap_or(false);
    let reply_count = c["replyCount"].as_i64().unwrap_or(0) as i32;

    let be_replied = c["beReplied"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|r| CommentReply {
                    userId: r["user"]["userId"].as_i64().unwrap_or(0),
                    nickname: r["user"]["nickname"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    content: r["content"].as_str().unwrap_or_default().to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    let user_id = c["user"]["userId"].as_i64().unwrap_or(0);
    let time_ms = c["time"].as_i64().unwrap_or(0);
    let parent_comment_id = c["parentCommentId"].as_i64().unwrap_or(0);
    // v2 comments carry `ipLocation` as an object ({ip, location, userId});
    // older responses may use a plain string.
    let ip_location = if let Some(s) = c["ipLocation"].as_str() {
        s.to_string()
    } else {
        c["ipLocation"]["location"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };
    let is_hot = c["hot"].as_bool().unwrap_or(false);

    CommentItem {
        id,
        content,
        user,
        avatar,
        time,
        likedCount: liked_count,
        liked,
        replyCount: reply_count,
        beReplied: be_replied,
        userId: user_id,
        timeMs: time_ms,
        parentCommentId: parent_comment_id,
        ipLocation: ip_location,
        isHot: is_hot,
        topReplies: vec![],
    }
}

pub fn parse_video_item(v: &Value) -> VideoItem {
    let id = v["id"]
        .as_i64()
        .map(|n| n.to_string())
        .or_else(|| v["id"].as_str().map(String::from))
        .or_else(|| v["vid"].as_str().map(String::from))
        .unwrap_or_default();

    let title = v["name"]
        .as_str()
        .or_else(|| v["title"].as_str())
        .unwrap_or_default()
        .to_string();

    let cover_url = v["cover"]
        .as_str()
        .or_else(|| v["coverUrl"].as_str())
        .map(String::from);

    let duration_ms = v["duration"]
        .as_i64()
        .or_else(|| v["durationms"].as_i64())
        .unwrap_or(0);

    let creators = v["artists"]
        .as_array()
        .or_else(|| v["creator"].as_array())
        .map(|arr| arr.iter().map(parse_artist).collect())
        .unwrap_or_default();

    let description = v["desc"]
        .as_str()
        .or_else(|| v["description"].as_str())
        .map(String::from);

    let play_count = v["playCount"].as_i64();

    VideoItem {
        id,
        title,
        coverUrl: cover_url,
        durationMs: duration_ms,
        creators,
        description,
        playCount: play_count,
    }
}

pub fn parse_dj_radio(d: &Value) -> DjRadio {
    let id = d["id"].as_i64().map(|n| n.to_string()).unwrap_or_default();

    let title = d["name"].as_str().unwrap_or_default().to_string();
    let cover_url = d["picUrl"].as_str().map(String::from);
    let description = d["desc"].as_str().map(String::from);
    let creator = if d["dj"].is_object() {
        Some(parse_social_user(&d["dj"]))
    } else {
        None
    };
    let program_count = d["programCount"].as_i64().map(|n| n as i32);

    DjRadio {
        id,
        title,
        coverUrl: cover_url,
        description,
        creator,
        programCount: program_count,
    }
}

pub fn parse_dj_program(p: &Value) -> DjProgram {
    let id = p["id"].as_i64().map(|n| n.to_string()).unwrap_or_default();

    let title = p["name"].as_str().unwrap_or_default().to_string();
    let radio_id = p["radio"]["id"].as_i64().map(|n| n.to_string());
    let track = parse_track(&p["mainSong"]);
    let description = p["description"].as_str().map(String::from);
    let published_at = p["createTime"].as_i64();

    DjProgram {
        id,
        title,
        radioId: radio_id,
        track,
        description,
        publishedAtEpochMs: published_at,
    }
}

pub fn parse_banner(b: &Value) -> Banner {
    Banner {
        id: b["bannerId"]
            .as_str()
            .map(String::from)
            .or_else(|| b["targetId"].as_i64().map(|n| n.to_string()))
            .unwrap_or_default(),
        imageUrl: b["pic"]
            .as_str()
            .or_else(|| b["imageUrl"].as_str())
            .unwrap_or_default()
            .to_string(),
        title: b["typeTitle"].as_str().unwrap_or_default().to_string(),
        targetType: b["targetType"].as_i64().unwrap_or(0) as i32,
        targetId: b["targetId"]
            .as_i64()
            .map(|n| n.to_string())
            .or_else(|| b["targetId"].as_str().map(String::from))
            .unwrap_or_default(),
    }
}
