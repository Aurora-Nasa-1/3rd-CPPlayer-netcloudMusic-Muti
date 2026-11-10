//! Parsers for converting NetEase API JSON responses into SDK DTO models.

use crate::sdk::models::*;
use serde_json::Value;

pub fn parse_artist(ar: &Value) -> Artist {
    Artist {
        id: ar["id"].as_i64().map(|n| n.to_string()).or_else(|| ar["id"].as_str().map(String::from)),
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
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
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
            .or_else(|| u["id"].as_i64().map(|n| n.to_string()))
            .unwrap_or_default(),
        nickname: u["nickname"].as_str().unwrap_or_default().to_string(),
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
    let id = c["commentId"].as_i64().unwrap_or(0);
    let content = c["content"].as_str().unwrap_or_default().to_string();
    let user = c["user"]["nickname"].as_str().unwrap_or_default().to_string();
    let avatar = c["user"]["avatarUrl"].as_str().unwrap_or_default().to_string();
    let time = c["timeStr"].as_str().map(String::from).unwrap_or_else(|| c["time"].to_string());
    let liked_count = c["likedCount"].as_i64().unwrap_or(0) as i32;
    let liked = c["liked"].as_bool().unwrap_or(false);
    let reply_count = c["replyCount"].as_i64().unwrap_or(0) as i32;

    let be_replied = c["beReplied"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|r| CommentReply {
                    userId: r["user"]["userId"].as_i64().unwrap_or(0),
                    nickname: r["user"]["nickname"].as_str().unwrap_or_default().to_string(),
                    content: r["content"].as_str().unwrap_or_default().to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    let user_id = c["user"]["userId"].as_i64().unwrap_or(0);
    let time_ms = c["time"].as_i64().unwrap_or(0);
    let parent_comment_id = c["parentCommentId"].as_i64().unwrap_or(0);
    let ip_location = c["ipLocation"]["location"].as_str().unwrap_or_default().to_string();

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
        isHot: false,
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
    let id = d["id"]
        .as_i64()
        .map(|n| n.to_string())
        .unwrap_or_default();

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
    let id = p["id"]
        .as_i64()
        .map(|n| n.to_string())
        .unwrap_or_default();

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
