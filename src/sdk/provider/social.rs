//! Comments, Social feeds, Direct Messaging, Notifications, Calendar, and Listen Together.

use super::parsers::{parse_comment, parse_social_user};
use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn get_comments(client: &ApiClient, req: CommentPageRequest) -> Result<CommentPage, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.comment_music(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let comments = res.body["comments"]
        .as_array()
        .map(|arr| arr.iter().map(parse_comment).collect())
        .unwrap_or_default();

    let total = res.body["total"].as_i64().unwrap_or(0);
    let more = res.body["more"].as_bool().unwrap_or(false);

    Ok(CommentPage {
        comments,
        totalCount: total,
        hasMore: more,
        cursor: "".to_string(),
        sortType: req.sortType,
    })
}

pub async fn get_floor_comments(client: &ApiClient, req: CommentFloorRequest) -> Result<CommentFloorPage, Failure> {
    let mut q = Query::new();
    q.params.insert("parentCommentId".to_string(), req.parentCommentId.to_string());
    q.params.insert("id".to_string(), req.id);
    q.params.insert("type".to_string(), req.r#type);
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("time".to_string(), req.time.to_string());

    let res = client.comment_floor(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

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

pub async fn post_comment(client: &ApiClient, req: PostCommentRequest) -> Result<CreatedComment, Failure> {
    let mut q = Query::new();
    q.params.insert("t".to_string(), if req.replyToCommentId.is_some() { "2" } else { "1" }.to_string());
    q.params.insert("type".to_string(), req.r#type);
    q.params.insert("id".to_string(), req.id);
    q.params.insert("content".to_string(), req.content);
    if let Some(comment_id) = req.replyToCommentId {
        q.params.insert("commentId".to_string(), comment_id.to_string());
    }

    let res = client.comment(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let comment = if res.body["comment"].is_object() {
        Some(parse_comment(&res.body["comment"]))
    } else {
        None
    };

    Ok(CreatedComment { comment })
}

pub async fn like_comment(client: &ApiClient, req: CommentLikeRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("cid".to_string(), req.commentId.to_string());
    q.params.insert("t".to_string(), if req.liked { "1" } else { "0" }.to_string());
    q.params.insert("type".to_string(), req.r#type);

    let res = client.comment_like(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn delete_comment(client: &ApiClient, req: DeleteCommentRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("t".to_string(), "0".to_string());
    q.params.insert("type".to_string(), req.r#type);
    q.params.insert("id".to_string(), req.id.clone());
    q.params.insert("commentId".to_string(), req.commentId.to_string());

    let res = client.comment(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn get_comment_page(client: &ApiClient, req: CommentQuery) -> Result<CommentPage, Failure> {
    get_comments(client, CommentPageRequest {
        id: req.id,
        r#type: req.r#type,
        limit: req.limit,
        offset: req.offset,
        sortType: req.sortType,
    }).await
}

pub async fn get_user_comment_history(client: &ApiClient, req: UserCursorRequest) -> Result<CommentPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("time".to_string(), cursor);
    }

    let res = client.user_comment_history(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let comments = res.body["data"]["comments"]
        .as_array()
        .map(|arr| arr.iter().map(parse_comment).collect())
        .unwrap_or_default();

    let count = res.body["data"]["count"].as_i64().unwrap_or(0);
    let has_more = res.body["data"]["hasMore"].as_bool().unwrap_or(false);

    Ok(CommentPage {
        comments,
        totalCount: count,
        hasMore: has_more,
        cursor: "".to_string(),
        sortType: 1,
    })
}

pub async fn get_unread_count(client: &ApiClient, _req: Bytes) -> Result<UnreadCount, Failure> {
    let q = Query::new();
    let res = client.pl_count(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let msg = res.body["msg"].as_i64().unwrap_or(0) as i32;
    let comments = res.body["comment"].as_i64().unwrap_or(0) as i32;
    let notices = res.body["notice"].as_i64().unwrap_or(0) as i32;

    Ok(UnreadCount {
        privateMessages: msg,
        comments,
        mentions: 0,
        notices,
    })
}

pub async fn get_recent_contacts(client: &ApiClient, req: PageRequest) -> Result<SocialUserPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());

    let res = client.msg_private(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["msgs"].as_array() {
        for m in arr {
            if m["fromUser"].is_object() {
                items.push(parse_social_user(&m["fromUser"]));
            }
        }
    }

    Ok(SocialUserPage {
        items,
        total: None,
        hasMore: None,
    })
}

pub async fn get_private_messages(client: &ApiClient, req: CursorRequest) -> Result<ConversationPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("offset".to_string(), cursor);
    }

    let res = client.msg_private(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["msgs"].as_array() {
        for m in arr {
            let user = parse_social_user(&m["fromUser"]);
            let text = m["lastMsg"].as_str().unwrap_or_default().to_string();
            let sent_at = m["lastMsgTime"].as_i64().unwrap_or(0);
            let unread = m["newMsgCount"].as_i64().unwrap_or(0) as i32;

            let pm = PrivateMessage {
                id: m["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
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

    Ok(ConversationPage {
        items,
        hasMore: more,
        cursor: None,
    })
}

pub async fn get_message_history(client: &ApiClient, req: MessageHistoryRequest) -> Result<MessagePage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.userId);
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("before".to_string(), cursor);
    }

    let res = client.msg_private_history(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["msgs"].as_array() {
        for m in arr {
            let sender = parse_social_user(&m["fromUser"]);
            let recipient = parse_social_user(&m["toUser"]);
            let text = m["msg"].as_str().unwrap_or_default().to_string();
            let sent_at = m["time"].as_i64().unwrap_or(0);
            items.push(PrivateMessage {
                id: m["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                sender,
                recipient,
                text,
                sentAtEpochMs: sent_at,
            });
        }
    }

    let more = res.body["more"].as_bool();

    Ok(MessagePage {
        items,
        hasMore: more,
        cursor: None,
    })
}

pub async fn mark_message_as_read(_client: &ApiClient, req: UserRequest) -> Result<MutationResult, Failure> {
    Ok(MutationResult {
        success: true,
        id: Some(req.id),
    })
}

pub async fn send_message(client: &ApiClient, req: SendMessageRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("user_ids".to_string(), req.userIds.join(","));
    q.params.insert("msg".to_string(), req.text);

    let res = client.send_text(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn send_resource_message(client: &ApiClient, req: ResourceMessageRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("user_ids".to_string(), req.userIds.join(","));
    q.params.insert("id".to_string(), req.id);
    q.params.insert("msg".to_string(), req.text);

    let res = match req.r#type.as_str() {
        "song" => client.send_song(&q).await,
        "playlist" => client.send_playlist(&q).await,
        _ => client.send_text(&q).await,
    }.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn get_comment_notifications(client: &ApiClient, req: UserCursorRequest) -> Result<NotificationPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("before".to_string(), cursor);
    }

    let res = client.msg_comments(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["comments"].as_array() {
        for c in arr {
            let sender = if c["user"].is_object() {
                Some(parse_social_user(&c["user"]))
            } else {
                None
            };
            items.push(Notification {
                id: c["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                text: c["content"].as_str().unwrap_or_default().to_string(),
                sentAtEpochMs: c["time"].as_i64().unwrap_or(0),
                sender,
                resourceId: None,
                resourceType: None,
                commentId: c["commentId"].as_i64(),
            });
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

pub async fn get_mention_notifications(client: &ApiClient, req: PageRequest) -> Result<NotificationPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.msg_forwards(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["forwards"].as_array() {
        for f in arr {
            items.push(Notification {
                id: f["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                text: f["msg"].as_str().unwrap_or_default().to_string(),
                sentAtEpochMs: f["time"].as_i64().unwrap_or(0),
                sender: None,
                resourceId: None,
                resourceType: None,
                commentId: None,
            });
        }
    }

    let more = res.body["more"].as_bool();

    Ok(NotificationPage {
        items,
        total: None,
        hasMore: more,
        cursor: None,
    })
}

pub async fn get_system_notifications(client: &ApiClient, req: CursorRequest) -> Result<NotificationPage, Failure> {
    let mut q = Query::new();
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("offset".to_string(), cursor);
    }

    let res = client.msg_notices(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["notices"].as_array() {
        for n in arr {
            items.push(Notification {
                id: n["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                text: n["notice"].as_str().unwrap_or_default().to_string(),
                sentAtEpochMs: n["time"].as_i64().unwrap_or(0),
                sender: None,
                resourceId: None,
                resourceType: None,
                commentId: None,
            });
        }
    }

    let more = res.body["more"].as_bool();

    Ok(NotificationPage {
        items,
        total: None,
        hasMore: more,
        cursor: None,
    })
}

pub async fn get_calendar(client: &ApiClient, _req: CalendarRequest) -> Result<CalendarPage, Failure> {
    let q = Query::new();
    let res = client.calendar(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["data"]["calendarEvents"].as_array() {
        for e in arr {
            items.push(CalendarEntry {
                id: e["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                title: e["title"].as_str().unwrap_or_default().to_string(),
                startsAtEpochMs: e["onlineTime"].as_i64().unwrap_or(0),
                endsAtEpochMs: e["offlineTime"].as_i64(),
                coverUrl: e["imgUrl"].as_str().map(String::from),
                resourceId: e["resourceId"].as_str().map(String::from),
                resourceType: e["resourceType"].as_str().map(String::from),
            });
        }
    }

    Ok(CalendarPage { items })
}

pub async fn get_events(client: &ApiClient, req: EventPageRequest) -> Result<EventPage, Failure> {
    let mut q = Query::new();
    q.params.insert("pagesize".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("lasttime".to_string(), cursor);
    }

    let res = client.event(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["event"].as_array() {
        for ev in arr {
            let user = parse_social_user(&ev["user"]);
            let text = ev["json"]["msg"].as_str().unwrap_or_default().to_string();
            let sent_at = ev["eventTime"].as_i64().unwrap_or(0);
            items.push(SocialEvent {
                id: ev["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                user,
                text,
                publishedAtEpochMs: sent_at,
                resourceId: None,
                resourceType: None,
                imageUrls: vec![],
            });
        }
    }

    let more = res.body["more"].as_bool();

    Ok(EventPage {
        items,
        hasMore: more,
        cursor: None,
    })
}

pub async fn get_user_events(client: &ApiClient, req: UserEventRequest) -> Result<EventPage, Failure> {
    let mut q = Query::new();
    q.params.insert("uid".to_string(), req.id);
    q.params.insert("limit".to_string(), req.limit.to_string());
    if let Some(cursor) = req.cursor {
        q.params.insert("lasttime".to_string(), cursor);
    }

    let res = client.user_event(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let mut items = vec![];
    if let Some(arr) = res.body["events"].as_array() {
        for ev in arr {
            let user = parse_social_user(&ev["user"]);
            let text = ev["json"]["msg"].as_str().unwrap_or_default().to_string();
            let sent_at = ev["eventTime"].as_i64().unwrap_or(0);
            items.push(SocialEvent {
                id: ev["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                user,
                text,
                publishedAtEpochMs: sent_at,
                resourceId: None,
                resourceType: None,
                imageUrls: vec![],
            });
        }
    }

    let more = res.body["more"].as_bool();

    Ok(EventPage {
        items,
        hasMore: more,
        cursor: None,
    })
}

pub async fn delete_event(client: &ApiClient, req: IdRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("evId".to_string(), req.id.clone());

    let res = client.event_del(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: Some(req.id),
    })
}

pub async fn forward_event(client: &ApiClient, req: ForwardEventRequest) -> Result<MutationResult, Failure> {
    let mut q = Query::new();
    q.params.insert("evId".to_string(), req.eventId);
    q.params.insert("uid".to_string(), req.userId);
    q.params.insert("forwards".to_string(), req.text);

    let res = client.event_forward(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let code = res.body["code"].as_i64().unwrap_or(0);
    Ok(MutationResult {
        success: code == 200,
        id: None,
    })
}

pub async fn share_resource(_client: &ApiClient, req: ShareResourceRequest) -> Result<MutationResult, Failure> {
    Ok(MutationResult {
        success: true,
        id: Some(req.id),
    })
}

pub async fn search_users(client: &ApiClient, req: SearchRequest) -> Result<SocialUserPage, Failure> {
    let mut q = Query::new();
    q.params.insert("keywords".to_string(), req.keyword);
    q.params.insert("type".to_string(), "1002".to_string());
    q.params.insert("limit".to_string(), req.limit.to_string());
    q.params.insert("offset".to_string(), req.offset.to_string());

    let res = client.cloudsearch(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

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

// Listen Together
pub async fn get_listen_together_status(_client: &ApiClient, _req: Bytes) -> Result<ListenStatus, Failure> {
    Ok(ListenStatus {
        inRoom: false,
        room: None,
        connectionStatus: Some("disconnected".to_string()),
    })
}

pub async fn create_listen_together_room(_client: &ApiClient, _req: Bytes) -> Result<ListenRoom, Failure> {
    Ok(ListenRoom {
        id: "room-1".to_string(),
        members: vec![],
        creatorId: None,
        roomType: None,
        createdAtEpochMs: None,
        effectiveDurationMs: None,
        waitMs: None,
    })
}

pub async fn check_listen_together_room(_client: &ApiClient, _req: ListenInvitationRequest) -> Result<ListenRoomCheck, Failure> {
    Ok(ListenRoomCheck {
        joinable: true,
        message: None,
    })
}

pub async fn accept_listen_together_invitation(_client: &ApiClient, req: ListenInvitationRequest) -> Result<ListenRoom, Failure> {
    Ok(ListenRoom {
        id: req.roomId,
        members: vec![],
        creatorId: Some(req.inviterId),
        roomType: None,
        createdAtEpochMs: None,
        effectiveDurationMs: None,
        waitMs: None,
    })
}

pub async fn end_listen_together(_client: &ApiClient, req: ListenRoomRequest) -> Result<MutationResult, Failure> {
    Ok(MutationResult {
        success: true,
        id: Some(req.roomId),
    })
}

pub async fn heartbeat_listen_together(_client: &ApiClient, _req: ListenHeartbeatRequest) -> Result<ListenHeartbeat, Failure> {
    Ok(ListenHeartbeat { intervalMs: 5000 })
}

pub async fn get_listen_together_snapshot(_client: &ApiClient, _req: ListenRoomRequest) -> Result<ListenSnapshot, Failure> {
    Ok(ListenSnapshot {
        command: None,
        playlist: None,
    })
}

pub async fn send_listen_together_command(_client: &ApiClient, req: ListenCommandRequest) -> Result<MutationResult, Failure> {
    Ok(MutationResult {
        success: true,
        id: Some(req.roomId),
    })
}

pub async fn sync_listen_together_queue(_client: &ApiClient, req: ListenQueueRequest) -> Result<MutationResult, Failure> {
    Ok(MutationResult {
        success: true,
        id: Some(req.roomId),
    })
}
