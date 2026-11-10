//! Comments, Social feeds, Direct Messaging, Notifications, Calendar, and Listen Together.

use super::parsers::{
    comment_sort_sdk, comment_sort_upstream, comment_type_code, extract_message_text, ncm_err,
    parse_comment, parse_event, parse_social_user,
};
use crate::api::Query;
use crate::error::NcmError;
use crate::request::ApiClient;
use crate::sdk::models::*;

fn unsupported(message: &str) -> Failure {
    Failure {
        category: "Unsupported".to_string(),
        message: message.to_string(),
    }
}

fn runtime(message: &str) -> Failure {
    Failure {
        category: "Runtime".to_string(),
        message: message.to_string(),
    }
}

/// Resolve the SDK resource-type name to the upstream numeric thread code.
fn require_type_code(t: &str) -> Result<&'static str, Failure> {
    comment_type_code(t)
        .ok_or_else(|| runtime(&format!("unsupported comment resource type: {}", t)))
}

/// Fetch comments through the offset-only legacy endpoints (time order).
async fn comments_via_legacy(
    client: &ApiClient,
    code: &str,
    id: &str,
    limit: i32,
    offset: i32,
) -> Result<serde_json::Value, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), id.to_string());
    q.params.insert("limit".to_string(), limit.to_string());
    q.params.insert("offset".to_string(), offset.to_string());

    // 各分支是不同的 impl Future，须在分支内 await 统一为 Result 再合并。
    let res = match code {
        "0" => client.comment_music(&q).await,
        "1" => client.comment_mv(&q).await,
        "2" => client.comment_playlist(&q).await,
        "3" => client.comment_album(&q).await,
        "4" => client.comment_dj(&q).await,
        "5" => client.comment_video(&q).await,
        "6" => {
            q.params
                .insert("threadId".to_string(), format!("A_EV_2_{}", id));
            client.comment_event(&q).await
        }
        _ => {
            return Err(unsupported(
                "comment resource type has no offset paging upstream",
            ))
        }
    }
    .map_err(ncm_err)?;

    Ok(res.body)
}

/// Fetch comments through `comment_new` with explicit sort + cursor semantics.
async fn comments_via_new(
    client: &ApiClient,
    code: &str,
    id: &str,
    limit: i32,
    cursor: &str,
    sort_type: i64,
) -> Result<serde_json::Value, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), id.to_string());
    q.params.insert("type".to_string(), code.to_string());
    q.params.insert("limit".to_string(), limit.to_string());
    q.params.insert("pageNo".to_string(), "1".to_string());
    q.params.insert("pageSize".to_string(), limit.to_string());
    q.params.insert("cursor".to_string(), cursor.to_string());
    q.params
        .insert("sortType".to_string(), sort_type.to_string());

    let res = client.comment_new(&q).await.map_err(ncm_err)?;
    Ok(res.body)
}

/// Build a CommentPage from a `comment_new` response body.
fn comment_page_from_new(body: &serde_json::Value) -> CommentPage {
    let data = &body["data"];
    let comments = data["comments"]
        .as_array()
        .map(|arr| arr.iter().map(parse_comment).collect())
        .unwrap_or_default();
    let total = data["totalCount"]
        .as_i64()
        .or_else(|| body["totalCount"].as_i64())
        .unwrap_or(0);
    let more = data["hasMore"]
        .as_bool()
        .or_else(|| body["hasMore"].as_bool())
        .unwrap_or(false);
    let cursor = data["cursor"]
        .as_str()
        .or_else(|| body["cursor"].as_str())
        .unwrap_or("")
        .to_string();
    let sort = data["sortType"]
        .as_i64()
        .or_else(|| body["sortType"].as_i64())
        .map(comment_sort_sdk)
        .unwrap_or(1);

    CommentPage {
        comments,
        totalCount: total,
        hasMore: more,
        cursor,
        sortType: sort,
    }
}

pub async fn get_comments(
    client: &ApiClient,
    req: CommentPageRequest,
) -> Result<CommentPage, Failure> {
    let code = require_type_code(&req.r#type)?;

    if req.sortType == 3 {
        // Latest-order is what the legacy offset endpoints return natively.
        let body = comments_via_legacy(client, code, &req.id, req.limit, req.offset).await?;
        let comments = body["comments"]
            .as_array()
            .map(|arr| arr.iter().map(parse_comment).collect())
            .unwrap_or_default();
        let total = body["total"].as_i64().unwrap_or(0);
        let more = body["more"].as_bool().unwrap_or(false);
        return Ok(CommentPage {
            comments,
            totalCount: total,
            hasMore: more,
            cursor: body["cursor"].as_str().unwrap_or("").to_string(),
            sortType: 3,
        });
    }

    // Recommended / hot are only available through comment_new, which pages by
    // an opaque cursor. Translate the item offset into the cursor form the
    // upstream uses for each sort ("offset" / "normalHot#offset").
    let upstream_sort = comment_sort_upstream(req.sortType);
    let cursor = if upstream_sort == 2 {
        format!("normalHot#{}", req.offset)
    } else {
        req.offset.to_string()
    };
    let body = comments_via_new(client, code, &req.id, req.limit, &cursor, upstream_sort).await?;
    Ok(comment_page_from_new(&body))
}

pub async fn get_floor_comments(
    client: &ApiClient,
    req: CommentFloorRequest,
) -> Result<CommentFloorPage, Failure> {
    let code = require_type_code(&req.r#type)?;

    let mut q = Query::new();
    q.params.insert(
        "parentCommentId".to_string(),
        req.parentCommentId.to_string(),
    );
    q.params.insert("id".to_string(), req.id);
    q.params.insert("type".to_string(), code.to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("time".to_string(), req.time.to_string());

    let res = client.comment_floor(&q).await.map_err(ncm_err)?;

    let replies = res.body["data"]["comments"]
        .as_array()
        .map(|arr| arr.iter().map(parse_comment).collect())
        .unwrap_or_default();

    let total = res.body["data"]["totalCount"].as_i64().unwrap_or(0) as i32;
    let has_more = res.body["data"]["hasMore"].as_bool().unwrap_or(false);
    let next_time = res.body["data"]["time"].as_i64().unwrap_or(0);

    Ok(CommentFloorPage {
        replies,
        totalCount: total,
        hasMore: has_more,
        nextTime: next_time,
    })
}

pub async fn post_comment(
    client: &ApiClient,
    req: PostCommentRequest,
) -> Result<CreatedComment, Failure> {
    let code = require_type_code(&req.r#type)?;

    let mut q = Query::new();
    q.params.insert(
        "t".to_string(),
        if req.replyToCommentId.is_some() {
            "2"
        } else {
            "1"
        }
        .to_string(),
    );
    q.params.insert("type".to_string(), code.to_string());
    q.params.insert("id".to_string(), req.id);
    q.params.insert("content".to_string(), req.content);
    if let Some(comment_id) = req.replyToCommentId {
        q.params
            .insert("commentId".to_string(), comment_id.to_string());
    }

    let res = client.comment(&q).await.map_err(ncm_err)?;

    let comment = if res.body["comment"].is_object() {
        Some(parse_comment(&res.body["comment"]))
    } else {
        None
    };

    Ok(CreatedComment { comment })
}

pub async fn like_comment(
    client: &ApiClient,
    req: CommentLikeRequest,
) -> Result<MutationResult, Failure> {
    let code = require_type_code(&req.r#type)?;

    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params
        .insert("cid".to_string(), req.commentId.to_string());
    q.params.insert(
        "t".to_string(),
        if req.liked { "1" } else { "0" }.to_string(),
    );
    q.params.insert("type".to_string(), code.to_string());

    let res = client.comment_like(&q).await.map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn delete_comment(
    client: &ApiClient,
    req: DeleteCommentRequest,
) -> Result<MutationResult, Failure> {
    let code = require_type_code(&req.r#type)?;

    let mut q = Query::new();
    q.params.insert("t".to_string(), "0".to_string());
    q.params.insert("type".to_string(), code.to_string());
    q.params.insert("id".to_string(), req.id.clone());
    q.params
        .insert("commentId".to_string(), req.commentId.to_string());

    let res = client.comment(&q).await.map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_comment_page(
    client: &ApiClient,
    req: CommentQuery,
) -> Result<CommentPage, Failure> {
    let code = require_type_code(&req.r#type)?;
    let upstream_sort = comment_sort_upstream(req.sortType);

    // Opaque cursors pass through verbatim (null and empty string are
    // distinct); when absent, translate the item offset into the cursor form
    // the upstream uses for the effective sort.
    let cursor = match req.cursor.as_deref() {
        Some(c) => c.to_string(),
        None => {
            if upstream_sort == 2 {
                format!("normalHot#{}", req.offset)
            } else if upstream_sort == 3 {
                "0".to_string()
            } else {
                req.offset.to_string()
            }
        }
    };

    let body = comments_via_new(client, code, &req.id, req.limit, &cursor, upstream_sort).await?;
    let mut page = comment_page_from_new(&body);
    // Response carries the effective server sort; keep the requested one when
    // the upstream echoes nothing.
    if page.sortType == 0 {
        page.sortType = req.sortType;
    }
    Ok(page)
}

pub async fn get_user_comment_history(
    client: &ApiClient,
    req: UserCursorRequest,
) -> Result<CommentPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("time".to_string(), cursor);
    }

    let res = client.user_comment_history(&q).await.map_err(ncm_err)?;

    let data = &res.body["data"];
    let comments: Vec<CommentItem> = data["comments"]
        .as_array()
        .map(|arr| arr.iter().map(parse_comment).collect())
        .unwrap_or_default();

    // Upstream totals this under `commentCount` (not `count`).
    let count = data["commentCount"]
        .as_i64()
        .or_else(|| res.body["commentCount"].as_i64())
        .unwrap_or(0);
    let has_more = data["hasMore"]
        .as_bool()
        .or_else(|| res.body["hasMore"].as_bool())
        .unwrap_or(false);
    // Continuation is the time cursor of the oldest returned comment.
    let cursor = comments
        .last()
        .map(|c| c.timeMs.to_string())
        .unwrap_or_default();

    Ok(CommentPage {
        comments,
        totalCount: count,
        hasMore: has_more,
        cursor,
        sortType: 3,
    })
}

pub async fn get_unread_count(client: &ApiClient, _req: Bytes) -> Result<UnreadCount, Failure> {
    let q = Query::new();
    let res = client.pl_count(&q).await.map_err(ncm_err)?;

    // Counts may be nested under `data` on some upstream revisions.
    let root = if res.body["data"].is_object() {
        &res.body["data"]
    } else {
        &res.body
    };
    let pick = |keys: &[&str]| -> i32 {
        keys.iter()
            .filter_map(|k| root[*k].as_i64())
            .next()
            .unwrap_or(0)
            .max(0) as i32
    };

    Ok(UnreadCount {
        privateMessages: pick(&["msg", "privateMsg", "msgPrivateCount"]),
        comments: pick(&["comment", "comments"]),
        // @-mentions / forwards are their own counter; never fake 0 from a
        // hard-coded constant.
        mentions: pick(&["forward", "forwards", "atMe", "@me", "mention"]),
        notices: pick(&["notice", "notices"]),
    })
}

fn contact_user(item: &serde_json::Value) -> serde_json::Value {
    for key in ["fromUser", "from", "user", "profile"] {
        if item[key].is_object() {
            return item[key].clone();
        }
    }
    item.clone()
}

fn contact_array<'a>(body: &'a serde_json::Value) -> Option<&'a Vec<serde_json::Value>> {
    body["recentcontacts"]
        .as_array()
        .or_else(|| body["msgs"].as_array())
        .or_else(|| body["data"]["recentcontacts"].as_array())
        .or_else(|| body["data"]["msgs"].as_array())
        .or_else(|| body["data"].as_array())
}

pub async fn get_recent_contacts(
    client: &ApiClient,
    req: PageRequest,
) -> Result<SocialUserPage, Failure> {
    let mut items = vec![];

    // Primary: msg/recentcontact. Fall back to the private-conversation list
    // when it yields nothing (established host convention: "empty" is the
    // fallback trigger, not just request failure).
    let q = Query::new();
    if let Ok(res) = client.msg_recentcontact(&q).await {
        if let Some(arr) = contact_array(&res.body) {
            for m in arr {
                let user = contact_user(m);
                if user["userId"].is_null() && user["id"].is_null() {
                    continue;
                }
                items.push(parse_social_user(&user));
            }
        }
    }

    if items.is_empty() {
        let mut q = Query::new();
        q.params.insert("limit".to_string(), req.limit.to_string());
        if req.offset > 0 {
            q.params
                .insert("offset".to_string(), req.offset.to_string());
        }
        let res = client.msg_private(&q).await.map_err(ncm_err)?;
        if let Some(arr) = contact_array(&res.body) {
            for m in arr {
                let user = contact_user(m);
                if user["userId"].is_null() && user["id"].is_null() {
                    continue;
                }
                items.push(parse_social_user(&user));
            }
        }
    }

    // Recent-contact paging metadata is not exposed by the upstream.
    Ok(SocialUserPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_private_messages(
    client: &ApiClient,
    req: CursorRequest,
) -> Result<ConversationPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    let offset = req
        .cursor
        .as_deref()
        .map(|c| c.parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
    if let Some(cursor) = req.cursor {
        // Preserve the caller's token verbatim (null vs empty stay distinct).
        q.params.insert("offset".to_string(), cursor);
    }

    let res = client.msg_private(&q).await.map_err(ncm_err)?;

    let mut items = vec![];
    if let Some(arr) = res.body["msgs"]
        .as_array()
        .or_else(|| res.body["data"]["msgs"].as_array())
    {
        for m in arr {
            let user = parse_social_user(&contact_user(m));
            let text = extract_message_text(&m["lastMsg"]);
            let sent_at = m["lastMsgTime"]
                .as_i64()
                .or_else(|| m["time"].as_i64())
                .unwrap_or(0);
            let unread = m["newMsgCount"].as_i64().unwrap_or(0) as i32;

            let pm = PrivateMessage {
                id: m["id"]
                    .as_i64()
                    .map(|n| n.to_string())
                    .or_else(|| m["id"].as_str().map(String::from))
                    .unwrap_or_default(),
                sender: user.clone(),
                recipient: user.clone(),
                text,
                sentAtEpochMs: sent_at,
            };

            items.push(Conversation {
                user,
                lastMessage: Some(pm),
                unreadCount: unread,
            });
        }
    }

    let more = res.body["more"].as_bool();
    // Offset continuation token for the next page.
    let cursor = if more.unwrap_or(false) {
        Some((offset + items.len() as i64).to_string())
    } else {
        None
    };

    Ok(ConversationPage {
        items,
        hasMore: more,
        cursor,
    })
}

pub async fn get_message_history(
    client: &ApiClient,
    req: MessageHistoryRequest,
) -> Result<MessagePage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.userId);
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("before".to_string(), cursor);
    }

    let res = client.msg_private_history(&q).await.map_err(ncm_err)?;

    let mut items = vec![];
    if let Some(arr) = res.body["msgs"]
        .as_array()
        .or_else(|| res.body["data"]["msgs"].as_array())
    {
        for m in arr {
            let sender = parse_social_user(&m["fromUser"]);
            let recipient = parse_social_user(&m["toUser"]);
            let text = extract_message_text(&m["msg"]);
            let sent_at = m["time"].as_i64().unwrap_or(0);
            items.push(PrivateMessage {
                id: m["id"]
                    .as_i64()
                    .map(|n| n.to_string())
                    .or_else(|| m["msgId"].as_i64().map(|n| n.to_string()))
                    .or_else(|| m["id"].as_str().map(String::from))
                    .unwrap_or_default(),
                sender,
                recipient,
                text,
                sentAtEpochMs: sent_at,
            });
        }
    }

    let more = res.body["more"].as_bool();
    // `before` paging continues from the oldest returned message; upstream
    // returns newest-first, so that is the last item's timestamp.
    let cursor = if more.unwrap_or(false) {
        items.last().map(|m| m.sentAtEpochMs.to_string())
    } else {
        None
    };

    Ok(MessagePage {
        items,
        hasMore: more,
        cursor,
    })
}

pub async fn mark_message_as_read(
    _client: &ApiClient,
    _req: UserRequest,
) -> Result<MutationResult, Failure> {
    // The upstream has no HTTP mark-read endpoint (`/api/msg/private/markread`
    // 404s; the official client marks reads over the Yunxin IM long-connection).
    // Faking success here would violate the contract's honesty rules, so report
    // the capability as unsupported instead.
    Err(unsupported(
        "marking private messages read is not available over the upstream HTTP API",
    ))
}

pub async fn send_message(
    client: &ApiClient,
    req: SendMessageRequest,
) -> Result<MutationResult, Failure> {
    if req.userIds.is_empty() || req.userIds.len() > 100 {
        return Err(runtime("sendMessage requires 1..100 recipient user IDs"));
    }

    let mut q = Query::new();
    q.params
        .insert("user_ids".to_string(), req.userIds.join(","));
    q.params.insert("msg".to_string(), req.text);

    let res = client.send_text(&q).await.map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn send_resource_message(
    client: &ApiClient,
    req: ResourceMessageRequest,
) -> Result<MutationResult, Failure> {
    if req.userIds.is_empty() || req.userIds.len() > 100 {
        return Err(runtime(
            "sendResourceMessage requires 1..100 recipient user IDs",
        ));
    }

    let mut q = Query::new();
    q.params
        .insert("user_ids".to_string(), req.userIds.join(","));
    q.params.insert("msg".to_string(), req.text);

    // `send_playlist` reads the id from `playlist`; the other senders read `id`.
    let res = match req.r#type.as_str() {
        "song" | "music" => {
            q.params.insert("id".to_string(), req.id);
            client.send_song(&q).await
        }
        "album" => {
            q.params.insert("id".to_string(), req.id);
            client.send_album(&q).await
        }
        "playlist" => {
            q.params.insert("playlist".to_string(), req.id);
            client.send_playlist(&q).await
        }
        other => {
            return Err(runtime(&format!(
                "unsupported resource message type: {}",
                other
            )))
        }
    }
    .map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

/// Build a Notification from one upstream message/comment/notice entry.
fn parse_notification(n: &serde_json::Value, text_key: &str) -> Notification {
    let sender = if n["user"].is_object() {
        Some(parse_social_user(&n["user"]))
    } else if n["fromUser"].is_object() {
        Some(parse_social_user(&n["fromUser"]))
    } else {
        None
    };
    Notification {
        id: n["commentId"]
            .as_i64()
            .or_else(|| n["id"].as_i64())
            .map(|x| x.to_string())
            .or_else(|| n["id"].as_str().map(String::from))
            .unwrap_or_default(),
        text: extract_message_text(&n[text_key]),
        sentAtEpochMs: n["time"]
            .as_i64()
            .or_else(|| n["createTime"].as_i64())
            .unwrap_or(0),
        sender,
        resourceId: n["resourceId"]
            .as_str()
            .map(String::from)
            .or_else(|| n["resourceId"].as_i64().map(|x| x.to_string())),
        resourceType: n["resourceType"].as_str().map(String::from),
        commentId: n["commentId"].as_i64().or_else(|| n["id"].as_i64()),
    }
}

/// Pick the continuation cursor for time-paged notification endpoints:
/// an upstream-provided token when present, else the oldest item's timestamp.
fn notification_cursor(body: &serde_json::Value, items: &[Notification]) -> Option<String> {
    if let Some(c) = body["cursor"].as_str() {
        return Some(c.to_string());
    }
    if let Some(t) = body["lasttime"].as_i64() {
        return Some(t.to_string());
    }
    items.last().map(|n| n.sentAtEpochMs.to_string())
}

pub async fn get_comment_notifications(
    client: &ApiClient,
    req: UserCursorRequest,
) -> Result<NotificationPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("before".to_string(), cursor);
    }

    let res = client.msg_comments(&q).await.map_err(ncm_err)?;

    let mut items = vec![];
    if let Some(arr) = res.body["comments"]
        .as_array()
        .or_else(|| res.body["data"]["comments"].as_array())
    {
        for c in arr {
            items.push(parse_notification(c, "content"));
        }
    }

    let more = res.body["more"].as_bool();
    let total = res.body["total"].as_i64();
    let cursor = notification_cursor(&res.body, &items);

    Ok(NotificationPage {
        items,
        total,
        hasMore: more,
        cursor,
    })
}

pub async fn get_mention_notifications(
    client: &ApiClient,
    req: PageRequest,
) -> Result<NotificationPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.msg_forwards(&q).await.map_err(ncm_err)?;

    let mut items = vec![];
    if let Some(arr) = res.body["forwards"]
        .as_array()
        .or_else(|| res.body["msgs"].as_array())
    {
        for f in arr {
            items.push(parse_notification(f, "msg"));
        }
    }

    let more = res.body["more"].as_bool();
    let total = res.body["total"].as_i64();

    Ok(NotificationPage {
        items,
        total,
        hasMore: more,
        cursor: None,
    })
}

pub async fn get_system_notifications(
    client: &ApiClient,
    req: CursorRequest,
) -> Result<NotificationPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    // Notices page on a `lasttime` cursor, not an item offset.
    if let Some(cursor) = req.cursor {
        q.params.insert("lasttime".to_string(), cursor);
    }

    let res = client.msg_notices(&q).await.map_err(ncm_err)?;

    let mut items = vec![];
    if let Some(arr) = res.body["notices"]
        .as_array()
        .or_else(|| res.body["data"]["notices"].as_array())
    {
        for n in arr {
            let mut note = parse_notification(n, "notice");
            if note.text.is_empty() {
                note.text = extract_message_text(&n["msg"]);
            }
            if note.text.is_empty() {
                note.text = extract_message_text(&n["content"]);
            }
            items.push(note);
        }
    }

    let more = res.body["more"].as_bool();
    let total = res.body["total"].as_i64();
    let cursor = notification_cursor(&res.body, &items);

    Ok(NotificationPage {
        items,
        total,
        hasMore: more,
        cursor,
    })
}

pub async fn get_calendar(
    client: &ApiClient,
    req: CalendarRequest,
) -> Result<CalendarPage, Failure> {
    let mut q = Query::new();
    if let Some(start) = req.startAtEpochMs {
        q.params.insert("startTime".to_string(), start.to_string());
    }
    if let Some(end) = req.endAtEpochMs {
        q.params.insert("endTime".to_string(), end.to_string());
    }

    let res = client.calendar(&q).await.map_err(ncm_err)?;

    let mut items = vec![];
    if let Some(arr) = res.body["data"]["calendarEvents"]
        .as_array()
        .or_else(|| res.body["calendarEvents"].as_array())
    {
        for e in arr {
            items.push(CalendarEntry {
                id: e["id"]
                    .as_i64()
                    .map(|n| n.to_string())
                    .or_else(|| e["id"].as_str().map(String::from))
                    .unwrap_or_default(),
                title: e["title"].as_str().unwrap_or_default().to_string(),
                startsAtEpochMs: e["onlineTime"].as_i64().unwrap_or(0),
                endsAtEpochMs: e["offlineTime"].as_i64(),
                coverUrl: e["imgUrl"].as_str().map(String::from),
                resourceId: e["resourceId"]
                    .as_str()
                    .map(String::from)
                    .or_else(|| e["resourceId"].as_i64().map(|n| n.to_string())),
                resourceType: e["resourceType"].as_str().map(String::from),
            });
        }
    }

    Ok(CalendarPage { items })
}

fn event_page_from(body: &serde_json::Value, arr_key: &str) -> EventPage {
    let mut items = vec![];
    if let Some(arr) = body[arr_key]
        .as_array()
        .or_else(|| body["event"].as_array())
        .or_else(|| body["events"].as_array())
        .or_else(|| body["data"][arr_key].as_array())
    {
        for ev in arr {
            items.push(parse_event(ev));
        }
    }

    let more = body["more"].as_bool();
    let cursor = if more.unwrap_or(false) {
        body["lasttime"]
            .as_i64()
            .map(|t| t.to_string())
            .or_else(|| items.last().map(|e| e.publishedAtEpochMs.to_string()))
    } else {
        None
    };

    EventPage {
        items,
        hasMore: more,
        cursor,
    }
}

pub async fn get_events(client: &ApiClient, req: EventPageRequest) -> Result<EventPage, Failure> {
    let mut q = Query::new();
    q.params
        .insert("pagesize".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("lasttime".to_string(), cursor);
    }

    let res = client.event(&q).await.map_err(ncm_err)?;
    Ok(event_page_from(&res.body, "event"))
}

pub async fn get_user_events(
    client: &ApiClient,
    req: UserEventRequest,
) -> Result<EventPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("lasttime".to_string(), cursor);
    }

    let res = client.user_event(&q).await.map_err(ncm_err)?;
    Ok(event_page_from(&res.body, "events"))
}

pub async fn delete_event(client: &ApiClient, req: IdRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("evId".to_string(), req.id.clone());

    let res = client.event_del(&q).await.map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn forward_event(
    client: &ApiClient,
    req: ForwardEventRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("evId".to_string(), req.eventId);
    q.params.insert("uid".to_string(), req.userId);
    q.params.insert("forwards".to_string(), req.text);

    let res = client.event_forward(&q).await.map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn share_resource(
    client: &ApiClient,
    req: ShareResourceRequest,
) -> Result<MutationResult, Failure> {
    // Map the SDK resource-type names onto the upstream share discriminators
    // here, not in a JSON envelope at the call site.
    let upstream_type = match req.r#type.as_str() {
        "music" | "song" => "song",
        "playlist" => "playlist",
        "mv" | "video" => "mv",
        "dj" | "program" => "dj",
        "album" => "album",
        "event" => "event",
        other => {
            return Err(runtime(&format!(
                "unsupported share resource type: {}",
                other
            )));
        }
    };

    let mut q = Query::new();
    q.params
        .insert("type".to_string(), upstream_type.to_string());
    q.params.insert("msg".to_string(), req.text);
    q.params.insert("id".to_string(), req.id.clone());

    let res = client.share_resource(&q).await.map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn search_users(
    client: &ApiClient,
    req: SearchRequest,
) -> Result<SocialUserPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "1002".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params
        .insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(ncm_err)?;

    let users = res.body["result"]["userprofiles"]
        .as_array()
        .map(|arr| arr.iter().map(parse_social_user).collect())
        .unwrap_or_default();

    let count = res.body["result"]["userprofileCount"].as_i64();

    Ok(SocialUserPage {
        items: users,
        total: count,
        hasMore: None,
    })
}

// ==================== Listen Together ====================
//
// Upstream response shapes are from the former KMP implementation
// (NeteaseListenTogetherBackend / ListenTogetherModels, commit 94a855a):
//   status:   {data:{inRoom, roomInfo, status}}
//   roomInfo: {roomId, creatorId, roomType, roomCreateTime, effectiveDurationMs,
//              waitMs, roomUsers[]{userId, nickname, avatarUrl}}
//   snapshot: {data:{playCommand, playlist}} (empty data => both null)
//   playlist: {displayList.result[], replace, version[]{userId, version}}
//   heartbeat response: timeSpan in **seconds** (measured 30)

fn parse_listen_member(u: &serde_json::Value) -> ListenMember {
    ListenMember {
        user: parse_social_user(u),
        // `roomUsers` lists the users currently occupying the room.
        online: true,
    }
}

fn parse_listen_room(o: &serde_json::Value) -> ListenRoom {
    let members = o["roomUsers"]
        .as_array()
        .map(|arr| arr.iter().map(parse_listen_member).collect())
        .unwrap_or_default();
    ListenRoom {
        id: o["roomId"]
            .as_str()
            .map(String::from)
            .or_else(|| o["roomId"].as_i64().map(|n| n.to_string()))
            .unwrap_or_default(),
        members,
        creatorId: o["creatorId"]
            .as_i64()
            .map(|n| n.to_string())
            .or_else(|| o["creatorId"].as_str().map(String::from))
            .filter(|s| !s.is_empty()),
        roomType: o["roomType"].as_str().map(String::from),
        createdAtEpochMs: o["roomCreateTime"].as_i64(),
        effectiveDurationMs: o["effectiveDurationMs"].as_i64(),
        waitMs: o["waitMs"].as_i64(),
    }
}

fn parse_listen_command(o: &serde_json::Value) -> ListenCommand {
    let action = o["commandType"]
        .as_str()
        .or_else(|| o["type"].as_str())
        .unwrap_or_default()
        .to_uppercase();
    let play_status = o["playStatus"].as_str().unwrap_or_default().to_uppercase();
    let track_id = o["targetSongId"]
        .as_str()
        .map(String::from)
        .or_else(|| o["targetSongId"].as_i64().map(|n| n.to_string()))
        .filter(|s| !s.is_empty());
    let former = o["formerSongId"]
        .as_str()
        .map(String::from)
        .or_else(|| o["formerSongId"].as_i64().map(|n| n.to_string()))
        .filter(|s| !s.is_empty());
    let user_id = o["userId"]
        .as_i64()
        .map(|n| n.to_string())
        .or_else(|| o["userId"].as_str().map(String::from))
        .filter(|s| !s.is_empty() && s != "0");
    ListenCommand {
        action,
        trackId: track_id,
        positionMs: o["progress"].as_i64().unwrap_or(0),
        sequence: o["clientSeq"].as_i64().unwrap_or(0),
        playStatus: play_status,
        formerTrackId: former,
        serverSequence: o["serverSeq"].as_i64().unwrap_or(0),
        userId: user_id,
    }
}

fn parse_listen_playlist(o: &serde_json::Value) -> ListenPlaylist {
    let track_ids = o["displayList"]["result"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    v.as_str()
                        .map(String::from)
                        .or_else(|| v.as_i64().map(|n| n.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    let versions = o["version"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let user_id = v["userId"]
                        .as_i64()
                        .map(|n| n.to_string())
                        .or_else(|| v["userId"].as_str().map(String::from))?;
                    Some(ListenQueueVersion {
                        userId: user_id,
                        version: v["version"].as_i64().unwrap_or(0),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    ListenPlaylist {
        trackIds: track_ids,
        replace: o["replace"].as_bool().unwrap_or(false),
        versions,
    }
}

fn room_from_body(body: &serde_json::Value) -> Option<ListenRoom> {
    let data = &body["data"];
    let info = if data["roomInfo"].is_object() {
        &data["roomInfo"]
    } else if data["roomId"].is_object() || data["roomId"].is_string() || data["roomId"].is_i64() {
        data
    } else if body["roomInfo"].is_object() {
        &body["roomInfo"]
    } else {
        return None;
    };
    let room = parse_listen_room(info);
    if room.id.is_empty() {
        None
    } else {
        Some(room)
    }
}

pub async fn get_listen_together_status(
    client: &ApiClient,
    _req: Bytes,
) -> Result<ListenStatus, Failure> {
    let q = Query::new();
    let res = client.listentogether_status(&q).await.map_err(ncm_err)?;

    let data = &res.body["data"];
    let in_room = data["inRoom"].as_bool().unwrap_or(false);
    let room = room_from_body(&res.body);
    if in_room && room.is_none() {
        return Err(Failure {
            category: "InvalidResponse".to_string(),
            message: "listen-together status reported in-room without room metadata".to_string(),
        });
    }
    // Upstream `status` (e.g. "NOT_CONNECTED") is provider-defined; pass it
    // through verbatim.
    let connection_status = data["status"]
        .as_str()
        .or_else(|| res.body["status"].as_str())
        .map(String::from);

    Ok(ListenStatus {
        inRoom: in_room,
        room,
        connectionStatus: connection_status,
    })
}

pub async fn create_listen_together_room(
    client: &ApiClient,
    _req: Bytes,
) -> Result<ListenRoom, Failure> {
    let q = Query::new();
    let res = client
        .listentogether_room_create(&q)
        .await
        .map_err(ncm_err)?;
    room_from_body(&res.body).ok_or_else(|| Failure {
        category: "InvalidResponse".to_string(),
        message: "create-room response is missing roomInfo".to_string(),
    })
}

pub async fn check_listen_together_room(
    client: &ApiClient,
    req: ListenInvitationRequest,
) -> Result<ListenRoomCheck, Failure> {
    let mut q = Query::new();
    q.params.insert("roomId".to_string(), req.roomId);

    match client.listentogether_room_check(&q).await {
        Ok(res) => {
            let data = &res.body["data"];
            let joinable = data["joinable"]
                .as_bool()
                .or_else(|| data["canJoin"].as_bool())
                .or_else(|| data["valid"].as_bool())
                .or_else(|| data["canAccept"].as_bool())
                // A 200 that carries the room is an affirmative answer.
                .unwrap_or_else(|| {
                    data["roomId"].is_string()
                        || data["roomId"].is_i64()
                        || data["roomInfo"].is_object()
                });
            let message = data["message"]
                .as_str()
                .or_else(|| data["msg"].as_str())
                .map(String::from)
                .filter(|s| !s.is_empty() && s != "success");
            Ok(ListenRoomCheck { joinable, message })
        }
        // Business rejection from the upstream means the invitation cannot be
        // joined; that is the answer, not a transport failure. Auth/network
        // problems stay real failures.
        Err(NcmError::Api { msg, .. }) => Ok(ListenRoomCheck {
            joinable: false,
            message: Some(msg),
        }),
        Err(NcmError::AuthRequired(msg)) => Ok(ListenRoomCheck {
            joinable: false,
            message: Some(msg),
        }),
        Err(e) => Err(ncm_err(e)),
    }
}

pub async fn accept_listen_together_invitation(
    client: &ApiClient,
    req: ListenInvitationRequest,
) -> Result<ListenRoom, Failure> {
    let mut q = Query::new();
    q.params.insert("roomId".to_string(), req.roomId.clone());
    q.params.insert("inviterId".to_string(), req.inviterId);

    let res = client.listentogether_accept(&q).await.map_err(ncm_err)?;

    if let Some(room) = room_from_body(&res.body) {
        return Ok(room);
    }

    // The accept response carries no room on some revisions; read membership
    // back from status rather than fabricating one from the request.
    let status = get_listen_together_status(client, Bytes::default()).await?;
    status.room.ok_or_else(|| Failure {
        category: "InvalidResponse".to_string(),
        message: "accepted the invitation but no room is visible in status".to_string(),
    })
}

pub async fn end_listen_together(
    client: &ApiClient,
    req: ListenRoomRequest,
) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("roomId".to_string(), req.roomId.clone());

    let res = client.listentogether_end(&q).await.map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.roomId),
    })
}

pub async fn heartbeat_listen_together(
    client: &ApiClient,
    req: ListenHeartbeatRequest,
) -> Result<ListenHeartbeat, Failure> {
    let mut q = Query::new();
    q.params.insert("roomId".to_string(), req.roomId);
    q.params
        .insert("songId".to_string(), req.trackId.unwrap_or_default());
    q.params.insert("playStatus".to_string(), req.playStatus);
    q.params
        .insert("progress".to_string(), req.positionMs.to_string());

    let res = client.listentogether_heatbeat(&q).await.map_err(ncm_err)?;

    // Upstream returns `timeSpan` in seconds (measured 30); the SDK wants a
    // positive millisecond interval.
    let seconds = res.body["timeSpan"]
        .as_i64()
        .or_else(|| res.body["data"]["timeSpan"].as_i64())
        .unwrap_or(30);
    let interval_ms = if seconds > 0 { seconds * 1000 } else { 30_000 };
    Ok(ListenHeartbeat {
        intervalMs: interval_ms,
    })
}

pub async fn get_listen_together_snapshot(
    client: &ApiClient,
    req: ListenRoomRequest,
) -> Result<ListenSnapshot, Failure> {
    let mut q = Query::new();
    q.params.insert("roomId".to_string(), req.roomId);

    let res = client
        .listentogether_sync_playlist_get(&q)
        .await
        .map_err(ncm_err)?;

    // An empty `data` object means both halves are absent; a present
    // `playlist` object with empty `displayList` is a meaningful empty queue.
    let data = &res.body["data"];
    let command = data["playCommand"]
        .as_object()
        .map(|_| parse_listen_command(&data["playCommand"]));
    let playlist = data["playlist"]
        .as_object()
        .map(|_| parse_listen_playlist(&data["playlist"]));

    Ok(ListenSnapshot { command, playlist })
}

pub async fn send_listen_together_command(
    client: &ApiClient,
    req: ListenCommandRequest,
) -> Result<MutationResult, Failure> {
    let action = req.command.action.to_uppercase();
    match action.as_str() {
        "PLAY" | "PAUSE" | "PROGRESS" | "GOTO" => {}
        other => {
            return Err(runtime(&format!(
                "unsupported listen-together command action: {}",
                other
            )))
        }
    }
    if action == "GOTO" && req.command.trackId.is_none() {
        return Err(runtime("GOTO listen-together command requires a track ID"));
    }

    let mut q = Query::new();
    q.params.insert("roomId".to_string(), req.roomId.clone());
    q.params.insert("commandType".to_string(), action);
    q.params
        .insert("progress".to_string(), req.command.positionMs.to_string());
    q.params
        .insert("playStatus".to_string(), req.command.playStatus);
    q.params.insert(
        "formerSongId".to_string(),
        req.command.formerTrackId.unwrap_or_default(),
    );
    q.params.insert(
        "targetSongId".to_string(),
        req.command.trackId.unwrap_or_default(),
    );
    q.params
        .insert("clientSeq".to_string(), req.command.sequence.to_string());

    let res = client
        .listentogether_play_command(&q)
        .await
        .map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.roomId),
    })
}

pub async fn sync_listen_together_queue(
    client: &ApiClient,
    req: ListenQueueRequest,
) -> Result<MutationResult, Failure> {
    let action = req.action.to_uppercase();
    match action.as_str() {
        "REPLACE" | "ADD" | "DELETE" => {}
        other => {
            return Err(runtime(&format!(
                "unsupported listen-together queue action: {}",
                other
            )))
        }
    }

    // Upstream splits randomList / displayList on commas; pass a joined
    // string. An empty REPLACE is a meaningful clear (empty joined string).
    let joined = req.trackIds.join(",");

    let mut q = Query::new();
    q.params.insert("roomId".to_string(), req.roomId.clone());
    q.params.insert("commandType".to_string(), action);
    q.params.insert("userId".to_string(), req.userId);
    q.params
        .insert("version".to_string(), req.version.to_string());
    q.params.insert("randomList".to_string(), joined.clone());
    q.params.insert("displayList".to_string(), joined);

    let res = client
        .listentogether_sync_list_command(&q)
        .await
        .map_err(ncm_err)?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.roomId),
    })
}
