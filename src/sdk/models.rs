// Generated from provider-contract/schema.mjs; do not edit.
#![allow(unused_imports, clippy::all, non_snake_case)]

use super::wire::{WireError, WireReader, WireWriter};
use serde::{Deserialize, Serialize};

/// A track/video creator with an optional provider-local identity.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Artist {
    /// Optional opaque artist ID; null means the upstream supplied a credit without an identity.
    pub id: Option<String>,
    /// Display name.
    pub name: String,
}

impl Artist {
    pub fn write(&self, w: &mut WireWriter) {
        w.optional(self.id.as_deref(), |w, s| w.string(s));
        w.string(&self.name);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.optional(|r| r.string())?,
            name: r.string()?,
        })
    }
}

/// Audio metadata. Unknown optional metadata stays null; duration is always milliseconds.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Track {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Ordered credited artists.
    pub artists: Vec<Artist>,
    /// Duration in milliseconds; never infer seconds from magnitude.
    pub durationMs: i64,
    /// Album name.
    pub album: Option<String>,
    /// Optional cover image URL.
    pub coverUrl: Option<String>,
}

impl Track {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.title);
        w.list(&self.artists, |w, item| item.write(w));
        w.long(self.durationMs);
        w.optional(self.album.as_deref(), |w, s| w.string(s));
        w.optional(self.coverUrl.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            title: r.string()?,
            artists: r.list(|r| Artist::read(r))?,
            durationMs: r.long()?,
            album: r.optional(|r| r.string())?,
            coverUrl: r.optional(|r| r.string())?,
        })
    }
}

/// Keyword search with bounded item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Search text, not an encoded query string.
    pub keyword: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl SearchRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.keyword);
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            keyword: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// A page of tracks with optional total and continuation metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrackPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Track>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
}

impl TrackPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Track::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
        })
    }
}

/// Audio resolution input using a provider-defined quality key.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResolveRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Provider-defined requested quality key; do not report it as actual quality without verification.
    pub quality: String,
}

impl ResolveRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.quality);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            quality: r.string()?,
        })
    }
}

/// One explicit HTTP header. Resource headers must not implicitly inherit account credentials.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Header {
    /// HTTP token header name; media-resource names must be unique case-insensitively.
    pub name: String,
    /// Header text; resource headers reject CR, LF and NUL and are bounded to 8192 characters.
    pub value: String,
}

impl Header {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.name);
        w.string(&self.value);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            name: r.string()?,
            value: r.string()?,
        })
    }
}

/// Resolved playable resource with actual quality, explicit headers and optional measured metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MediaResource {
    /// Absolute resource URL.
    pub url: String,
    /// Actual resolved quality key, which may differ from the requested key.
    pub actualQuality: String,
    /// Explicit headers; no implicit account Cookie or Authorization forwarding.
    pub headers: Vec<Header>,
    /// Unix epoch milliseconds when access expires; null means unknown.
    pub expiresAtEpochMs: Option<i64>,
    /// Actual container/encoding description when known.
    pub format: Option<String>,
    /// Bitrate in bits per second, not kilobits per second.
    pub bitrate: Option<i32>,
    /// Byte count; null means unknown where nullable.
    pub sizeBytes: Option<i64>,
}

impl MediaResource {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.url);
        w.string(&self.actualQuality);
        w.list(&self.headers, |w, item| item.write(w));
        w.optional(self.expiresAtEpochMs.as_ref(), |w, &v| w.long(v));
        w.optional(self.format.as_deref(), |w, s| w.string(s));
        w.optional(self.bitrate.as_ref(), |w, &v| w.int(v));
        w.optional(self.sizeBytes.as_ref(), |w, &v| w.long(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            url: r.string()?,
            actualQuality: r.string()?,
            headers: r.list(|r| Header::read(r))?,
            expiresAtEpochMs: r.optional(|r| r.long())?,
            format: r.optional(|r| r.string())?,
            bitrate: r.optional(|r| r.int())?,
            sizeBytes: r.optional(|r| r.long())?,
        })
    }
}

/// Sanitized, categorized business failure; never include credentials or private request bodies.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Failure {
    /// Unsupported, NotFound, Authentication, Network, RateLimit, Timeout, InvalidResponse or Runtime.
    pub category: String,
    /// Sanitized human-readable explanation, not a credential or upstream response dump.
    pub message: String,
}

impl Failure {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.category);
        w.string(&self.message);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            category: r.string()?,
            message: r.string()?,
        })
    }
}

/// One batch input's result; exactly one of track or failure is present.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrackOutcome {
    /// Must equal the input ID at the same batch position, including duplicate IDs.
    pub id: String,
    /// Exactly one of track or failure must be present; track.id must match this outcome's id.
    pub track: Option<Track>,
    /// Exactly one of track or failure must be present.
    pub failure: Option<Failure>,
}

impl TrackOutcome {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.optional(self.track.as_ref(), |w, item| item.write(w));
        w.optional(self.failure.as_ref(), |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            track: r.optional(|r| Track::read(r))?,
            failure: r.optional(|r| Failure::read(r))?,
        })
    }
}

/// Ordered outcomes, preserving the input IDs and duplicates.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrackBatch {
    /// Exactly one outcome per requested ID, in the same order and retaining duplicates.
    pub items: Vec<TrackOutcome>,
}

impl TrackBatch {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| TrackOutcome::read(r))?,
        })
    }
}

/// An exact provider-local resource identity; the operation determines its resource kind.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IdRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
}

impl IdRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
        })
    }
}

/// An ordered batch of opaque IDs; duplicates are meaningful.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Ordered provider-local IDs. Preserve order and duplicates.
    pub ids: Vec<String>,
}

impl BatchRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.ids, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            ids: r.list(|r| r.string())?,
        })
    }
}

/// Runtime method capabilities, fixed for one immutable account session.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Capabilities {
    /// Exact runtime capability strings, at most 256, fixed within a session.
    pub supported: Vec<String>,
}

impl Capabilities {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.supported, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            supported: r.list(|r| r.string())?,
        })
    }
}

/// Allowed-origin runtime HTTP request, not a browser fetch request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Absolute resource URL.
    pub url: String,
    /// Explicit HTTP method.
    pub method: String,
    /// Explicit headers; no implicit account Cookie or Authorization forwarding.
    pub headers: Vec<Header>,
    /// Binary HTTP body, not text or base64; use UTF-8 helpers when appropriate.
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.url);
        w.string(&self.method);
        w.list(&self.headers, |w, item| item.write(w));
        w.optional(self.body.as_deref(), |w, b| w.bytes(b));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            url: r.string()?,
            method: r.string()?,
            headers: r.list(|r| Header::read(r))?,
            body: r.optional(|r| r.bytes())?,
        })
    }
}

/// Runtime HTTP status, headers and binary response body.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code from the allowed-origin runtime call.
    pub status: i32,
    /// Explicit headers; no implicit account Cookie or Authorization forwarding.
    pub headers: Vec<Header>,
    /// Binary HTTP body, not text or base64; use UTF-8 helpers when appropriate.
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn write(&self, w: &mut WireWriter) {
        w.int(self.status);
        w.list(&self.headers, |w, item| item.write(w));
        w.bytes(&self.body);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            status: r.int()?,
            headers: r.list(|r| Header::read(r))?,
            body: r.bytes()?,
        })
    }
}

/// Binary wrapper; no-argument business operations require an empty value.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Bytes {
    /// Raw bytes. Pass an empty array for no-argument business operations.
    pub value: Vec<u8>,
}

impl Bytes {
    pub fn write(&self, w: &mut WireWriter) {
        w.bytes(&self.value);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            value: r.bytes()?,
        })
    }
}

/// Session-local temporary storage access; a null value deletes a key.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StorageRequest {
    /// Operation-specific storage/authentication key.
    pub key: String,
    /// Bytes to store; null deletes the key. Not persistent across account-session replacement.
    pub value: Option<Vec<u8>>,
}

impl StorageRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.key);
        w.optional(self.value.as_deref(), |w, b| w.bytes(b));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            key: r.string()?,
            value: r.optional(|r| r.bytes())?,
        })
    }
}

/// Stored session-local bytes, or null when absent.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StorageValue {
    /// Stored bytes or null when the key is absent.
    pub value: Option<Vec<u8>>,
}

impl StorageValue {
    pub fn write(&self, w: &mut WireWriter) {
        w.optional(self.value.as_deref(), |w, b| w.bytes(b));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            value: r.optional(|r| r.bytes())?,
        })
    }
}

/// An operation-specific resource ID with bounded item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PagedIdRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl PagedIdRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// A bounded recommendation/discovery selection, not offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RecommendationsRequest {
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
}

impl RecommendationsRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.int(self.limit);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            limit: r.int()?,
        })
    }
}

/// Playlist category and order selectors with a bounded selection size.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistBrowseRequest {
    /// Provider-supported category selector.
    pub category: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Provider-supported ordering selector.
    pub order: String,
}

impl PlaylistBrowseRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.category);
        w.int(self.limit);
        w.string(&self.order);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            category: r.string()?,
            limit: r.int()?,
            order: r.string()?,
        })
    }
}

/// Legacy album discovery selectors; use AlbumPageRequest for explicit offsets.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AlbumBrowseRequest {
    /// Provider-supported area selector; do not silently reinterpret unknown values.
    pub area: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
}

impl AlbumBrowseRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.area);
        w.int(self.limit);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            area: r.string()?,
            limit: r.int()?,
        })
    }
}

/// Track discovery with a provider-supported numeric region selector.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TopSongsRequest {
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// One of 0, 7, 8, 16 or 96; unsupported regional mappings must fail explicitly.
    pub region: i32,
}

impl TopSongsRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.int(self.limit);
        w.int(self.region);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            limit: r.int()?,
            region: r.int()?,
        })
    }
}

/// Provider-local user identity, not the host's account-slot key.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
}

impl UserRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
        })
    }
}

/// User listening-record selectors and a bounded result count.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserRecordsRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// 0 = all-time records, 1 = weekly records.
    #[serde(rename = "type")]
    pub r#type: i32,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
}

impl UserRecordsRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.r#type);
        w.int(self.limit);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            r#type: r.int()?,
            limit: r.int()?,
        })
    }
}

/// User-scoped item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserListRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl UserListRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// Public user profile and known counters.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserProfile {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Public display nickname.
    pub nickname: String,
    /// Optional avatar image URL.
    pub avatarUrl: Option<String>,
    /// Profile signature text; null means unchanged for patch requests.
    pub signature: Option<String>,
    /// Known playlist count.
    pub playlistCount: i32,
    /// Known following count.
    pub follows: i32,
    /// Known follower count.
    pub followeds: i32,
}

impl UserProfile {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.nickname);
        w.optional(self.avatarUrl.as_deref(), |w, s| w.string(s));
        w.optional(self.signature.as_deref(), |w, s| w.string(s));
        w.int(self.playlistCount);
        w.int(self.follows);
        w.int(self.followeds);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            nickname: r.string()?,
            avatarUrl: r.optional(|r| r.string())?,
            signature: r.optional(|r| r.string())?,
            playlistCount: r.int()?,
            follows: r.int()?,
            followeds: r.int()?,
        })
    }
}

/// A bounded ordered list of opaque IDs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IdList {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<String>,
}

impl IdList {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| r.string())?,
        })
    }
}

/// A bounded page of opaque IDs with explicit continuation; each item is a provider-local track ID.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IdPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<String>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: bool,
}

impl IdPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, s| w.string(s));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.bool(self.hasMore);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| r.string())?,
            total: r.optional(|r| r.long())?,
            hasMore: r.bool()?,
        })
    }
}

/// Mutation acknowledgement with an optional created/affected resource ID.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MutationResult {
    /// Explicit mutation/authentication acknowledgement; false is not an empty successful data page.
    pub success: bool,
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: Option<String>,
}

impl MutationResult {
    pub fn write(&self, w: &mut WireWriter) {
        w.bool(self.success);
        w.optional(self.id.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            success: r.bool()?,
            id: r.optional(|r| r.string())?,
        })
    }
}

/// A playlist identity and ordered provider-local track IDs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistTracksRequest {
    /// Opaque playlist identity.
    pub playlistId: String,
    /// Ordered audio-track identities.
    pub trackIds: Vec<String>,
}

impl PlaylistTracksRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.playlistId);
        w.list(&self.trackIds, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            playlistId: r.string()?,
            trackIds: r.list(|r| r.string())?,
        })
    }
}

/// Playlist rename input.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistNameRequest {
    /// Opaque playlist identity.
    pub playlistId: String,
    /// Display name.
    pub name: String,
}

impl PlaylistNameRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.playlistId);
        w.string(&self.name);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            playlistId: r.string()?,
            name: r.string()?,
        })
    }
}

/// Explicit desired playlist subscription state.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistSubscribeRequest {
    /// Opaque playlist identity.
    pub playlistId: String,
    /// Explicit desired subscription state, not a toggle.
    pub subscribed: bool,
}

impl PlaylistSubscribeRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.playlistId);
        w.bool(self.subscribed);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            playlistId: r.string()?,
            subscribed: r.bool()?,
        })
    }
}

/// Playlist creation input with a provider-supported privacy selector.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CreatePlaylistRequest {
    /// Display name.
    pub name: String,
    /// Provider-supported playlist privacy selector.
    pub privacy: i32,
}

impl CreatePlaylistRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.name);
        w.int(self.privacy);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            name: r.string()?,
            privacy: r.int()?,
        })
    }
}

/// Explicit desired liked state for an audio track.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SongLikeRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Explicit desired/current liked state, not a toggle.
    pub liked: bool,
}

impl SongLikeRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.bool(self.liked);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            liked: r.bool()?,
        })
    }
}

/// Compatibility comment-reply attribution; numeric user IDs are an existing v1 limitation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentReply {
    /// Existing v1 numeric user ID; a string-ID redesign requires a new compatible operation/record.
    pub userId: i64,
    /// Public display nickname.
    pub nickname: String,
    /// Plain-text content.
    pub content: String,
}

impl CommentReply {
    pub fn write(&self, w: &mut WireWriter) {
        w.long(self.userId);
        w.string(&self.nickname);
        w.string(&self.content);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            userId: r.long()?,
            nickname: r.string()?,
            content: r.string()?,
        })
    }
}

/// Comment metadata with optional inline floor seeds. Numeric comment/user IDs are an existing v1 limitation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentItem {
    /// Existing v1 positive numeric comment ID within JavaScript safe-integer bounds.
    pub id: i64,
    /// Plain-text content.
    pub content: String,
    /// Comment author's display nickname, not a SocialUser record.
    pub user: String,
    /// Comment avatar display URL/text preserved by the compatibility record.
    pub avatar: String,
    /// Display time text.
    pub time: String,
    /// Known nonnegative like count.
    pub likedCount: i32,
    /// Explicit desired/current liked state, not a toggle.
    pub liked: bool,
    /// Known nonnegative reply count.
    pub replyCount: i32,
    /// Reply attribution preserved from upstream.
    pub beReplied: Vec<CommentReply>,
    /// Existing v1 numeric user ID within JavaScript safe-integer bounds.
    pub userId: i64,
    /// Comment time in Unix epoch milliseconds.
    pub timeMs: i64,
    /// Numeric top-level comment identity.
    pub parentCommentId: i64,
    /// Public location attribution text.
    pub ipLocation: String,
    /// Whether upstream marks the comment as hot.
    pub isHot: bool,
    /// Inline floor seeds, not proof that a complete floor page has been fetched.
    pub topReplies: Vec<CommentItem>,
}

impl CommentItem {
    pub fn write(&self, w: &mut WireWriter) {
        w.long(self.id);
        w.string(&self.content);
        w.string(&self.user);
        w.string(&self.avatar);
        w.string(&self.time);
        w.int(self.likedCount);
        w.bool(self.liked);
        w.int(self.replyCount);
        w.list(&self.beReplied, |w, item| item.write(w));
        w.long(self.userId);
        w.long(self.timeMs);
        w.long(self.parentCommentId);
        w.string(&self.ipLocation);
        w.bool(self.isHot);
        w.list(&self.topReplies, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.long()?,
            content: r.string()?,
            user: r.string()?,
            avatar: r.string()?,
            time: r.string()?,
            likedCount: r.int()?,
            liked: r.bool()?,
            replyCount: r.int()?,
            beReplied: r.list(|r| CommentReply::read(r))?,
            userId: r.long()?,
            timeMs: r.long()?,
            parentCommentId: r.long()?,
            ipLocation: r.string()?,
            isHot: r.bool()?,
            topReplies: r.list(|r| CommentItem::read(r))?,
        })
    }
}

/// Comments, total and effective sort plus the next opaque cursor.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentPage {
    /// Ordered comments bounded by the request limit.
    pub comments: Vec<CommentItem>,
    /// Known total count, at least the number of items returned.
    pub totalCount: i64,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: bool,
    /// Next opaque token. When comments.page is used, an empty token is a real value, not null.
    pub cursor: String,
    /// 1 = recommended, 2 = hot, 3 = latest. Responses carry the effective server sort.
    pub sortType: i32,
}

impl CommentPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.comments, |w, item| item.write(w));
        w.long(self.totalCount);
        w.bool(self.hasMore);
        w.string(&self.cursor);
        w.int(self.sortType);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            comments: r.list(|r| CommentItem::read(r))?,
            totalCount: r.long()?,
            hasMore: r.bool()?,
            cursor: r.string()?,
            sortType: r.int()?,
        })
    }
}

/// Replies under one top-level comment and the next floor cursor.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentFloorPage {
    /// Ordered floor replies bounded by the request limit.
    pub replies: Vec<CommentItem>,
    /// Known total count, at least the number of items returned.
    pub totalCount: i32,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: bool,
    /// Numeric opaque floor cursor; echo unchanged, never convert units.
    pub nextTime: i64,
}

impl CommentFloorPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.replies, |w, item| item.write(w));
        w.int(self.totalCount);
        w.bool(self.hasMore);
        w.long(self.nextTime);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            replies: r.list(|r| CommentItem::read(r))?,
            totalCount: r.int()?,
            hasMore: r.bool()?,
            nextTime: r.long()?,
        })
    }
}

/// Existing offset-only comment input; its wire layout remains unchanged.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentPageRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Resource kind supported by the provider's offset-only comment operation.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
    /// 1 = recommended, 2 = hot, 3 = latest. Responses carry the effective server sort.
    pub sortType: i32,
}

impl CommentPageRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.r#type);
        w.int(self.limit);
        w.int(self.offset);
        w.int(self.sortType);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            r#type: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
            sortType: r.int()?,
        })
    }
}

/// Floor-comment input with a numeric continuation cursor.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentFloorRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Numeric top-level comment identity.
    pub parentCommentId: i64,
    /// Operation-specific discriminator; see the operation and record-specific constraints.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Numeric floor cursor; use 0 for the first page and echo nextTime thereafter.
    pub time: i64,
}

impl CommentFloorRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.long(self.parentCommentId);
        w.string(&self.r#type);
        w.int(self.limit);
        w.long(self.time);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            parentCommentId: r.long()?,
            r#type: r.string()?,
            limit: r.int()?,
            time: r.long()?,
        })
    }
}

/// New top-level comment or reply input.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PostCommentRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Operation-specific discriminator; see the operation and record-specific constraints.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Nonblank comment text, at most 4096 UTF-16 code units at the host boundary; the application applies its own editor limit.
    pub content: String,
    /// Numeric target comment identity; null creates a top-level comment.
    pub replyToCommentId: Option<i64>,
}

impl PostCommentRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.r#type);
        w.string(&self.content);
        w.optional(self.replyToCommentId.as_ref(), |w, &v| w.long(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            r#type: r.string()?,
            content: r.string()?,
            replyToCommentId: r.optional(|r| r.long())?,
        })
    }
}

/// Explicit desired liked state for a comment within a resource.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentLikeRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Existing v1 numeric comment identity; must stay within JavaScript safe-integer bounds.
    pub commentId: i64,
    /// Operation-specific discriminator; see the operation and record-specific constraints.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Explicit desired/current liked state, not a toggle.
    pub liked: bool,
}

impl CommentLikeRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.long(self.commentId);
        w.string(&self.r#type);
        w.bool(self.liked);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            commentId: r.long()?,
            r#type: r.string()?,
            liked: r.bool()?,
        })
    }
}

/// Successful creation may omit the created comment when upstream returns no body.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CreatedComment {
    /// Created comment, or null when a successful upstream mutation returns no comment body.
    pub comment: Option<CommentItem>,
}

impl CreatedComment {
    pub fn write(&self, w: &mut WireWriter) {
        w.optional(self.comment.as_ref(), |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            comment: r.optional(|r| CommentItem::read(r))?,
        })
    }
}

/// Seed audio identity with a bounded related-track selection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SimilarSongsRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
}

impl SimilarSongsRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.limit);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            limit: r.int()?,
        })
    }
}

/// Seed audio/playlist identities and a bounded intelligent selection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IntelligenceSongsRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Opaque playlist identity.
    pub playlistId: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
}

impl IntelligenceSongsRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.playlistId);
        w.int(self.limit);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            playlistId: r.string()?,
            limit: r.int()?,
        })
    }
}

/// Explicit desired album subscription state.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AlbumSubscribeRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Explicit desired subscription state, not a toggle.
    pub subscribed: bool,
}

impl AlbumSubscribeRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.bool(self.subscribed);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            subscribed: r.bool()?,
        })
    }
}

/// Search prefix/keyword with a bounded suggestion count.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SearchSuggestionRequest {
    /// Search text, not an encoded query string.
    pub keyword: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
}

impl SearchSuggestionRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.keyword);
        w.int(self.limit);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            keyword: r.string()?,
            limit: r.int()?,
        })
    }
}

/// Discovery keyword and descriptive text.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Keyword {
    /// Search text, not an encoded query string.
    pub keyword: String,
    /// Descriptive text; null means unavailable or unchanged according to the record.
    pub description: String,
}

impl Keyword {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.keyword);
        w.string(&self.description);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            keyword: r.string()?,
            description: r.string()?,
        })
    }
}

/// A bounded list of discovery keywords.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct KeywordPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Keyword>,
}

impl KeywordPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Keyword::read(r))?,
        })
    }
}

/// Playback report with elapsed whole seconds, not milliseconds.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ScrobbleRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Provider-local playback source identity.
    pub sourceId: String,
    /// Played whole seconds; this field intentionally does not use milliseconds.
    pub playedSeconds: i32,
}

impl ScrobbleRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.sourceId);
        w.int(self.playedSeconds);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            sourceId: r.string()?,
            playedSeconds: r.int()?,
        })
    }
}

/// Current login state without mutating the session's credential snapshot.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AccountSession {
    /// Whether the immutable account session is authenticated.
    pub loggedIn: bool,
    /// Opaque provider-local user identity, not the host account-slot key.
    pub userId: Option<String>,
    /// Public display nickname.
    pub nickname: Option<String>,
    /// Optional avatar image URL.
    pub avatarUrl: Option<String>,
}

impl AccountSession {
    pub fn write(&self, w: &mut WireWriter) {
        w.bool(self.loggedIn);
        w.optional(self.userId.as_deref(), |w, s| w.string(s));
        w.optional(self.nickname.as_deref(), |w, s| w.string(s));
        w.optional(self.avatarUrl.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            loggedIn: r.bool()?,
            userId: r.optional(|r| r.string())?,
            nickname: r.optional(|r| r.string())?,
            avatarUrl: r.optional(|r| r.string())?,
        })
    }
}

/// Named authentication workflow input; only fields relevant to action should be supplied.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AuthActionRequest {
    /// One of AuthAction's documented action names; required fields depend on the selected flow.
    pub action: String,
    /// Authentication email input; never log.
    pub email: Option<String>,
    /// Digits-only phone input; extended binding requests require 4..20 digits.
    pub phone: Option<String>,
    /// Authentication secret; never log, persist in diagnostics or expose in media headers.
    pub password: Option<String>,
    /// Captcha proof; do not log.
    pub captcha: Option<String>,
    /// Whether the supplied password is already MD5-hashed; this is not a file checksum.
    pub md5: bool,
    /// Image captcha answer.
    pub imageCaptcha: Option<String>,
    /// Authentication challenge credential; never log or forward to media resources.
    pub challengeCookie: Option<String>,
    /// Operation-specific storage/authentication key.
    pub key: Option<String>,
    /// Opaque authentication-flow session identity.
    pub sessionId: Option<String>,
    /// Opaque verification-flow identity.
    pub verifyId: Option<String>,
    /// Legacy authentication-only extension text, not a generic endpoint passthrough.
    pub extraJson: Option<String>,
}

impl AuthActionRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.action);
        w.optional(self.email.as_deref(), |w, s| w.string(s));
        w.optional(self.phone.as_deref(), |w, s| w.string(s));
        w.optional(self.password.as_deref(), |w, s| w.string(s));
        w.optional(self.captcha.as_deref(), |w, s| w.string(s));
        w.bool(self.md5);
        w.optional(self.imageCaptcha.as_deref(), |w, s| w.string(s));
        w.optional(self.challengeCookie.as_deref(), |w, s| w.string(s));
        w.optional(self.key.as_deref(), |w, s| w.string(s));
        w.optional(self.sessionId.as_deref(), |w, s| w.string(s));
        w.optional(self.verifyId.as_deref(), |w, s| w.string(s));
        w.optional(self.extraJson.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            action: r.string()?,
            email: r.optional(|r| r.string())?,
            phone: r.optional(|r| r.string())?,
            password: r.optional(|r| r.string())?,
            captcha: r.optional(|r| r.string())?,
            md5: r.bool()?,
            imageCaptcha: r.optional(|r| r.string())?,
            challengeCookie: r.optional(|r| r.string())?,
            key: r.optional(|r| r.string())?,
            sessionId: r.optional(|r| r.string())?,
            verifyId: r.optional(|r| r.string())?,
            extraJson: r.optional(|r| r.string())?,
        })
    }
}

/// Authentication result/challenge. The host owns credential storage and session replacement.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AuthActionResult {
    /// Explicit mutation/authentication acknowledgement; false is not an empty successful data page.
    pub success: bool,
    /// Provider authentication status code when known.
    pub code: Option<i32>,
    /// Provider authentication/check status code when known.
    pub status: Option<i32>,
    /// Operation-specific storage/authentication key.
    pub key: Option<String>,
    /// Absolute resource URL.
    pub url: Option<String>,
    /// Authentication captcha image payload, not an upload transport.
    pub imageBase64: Option<String>,
    /// New authentication credential for host storage; never copy into resource headers implicitly.
    pub cookie: Option<String>,
    /// Verification page URL.
    pub verifyUrl: Option<String>,
    /// Whether an additional authentication challenge is required.
    pub challenge: bool,
    /// Sanitized human-readable explanation, not a credential or upstream response dump.
    pub message: Option<String>,
}

impl AuthActionResult {
    pub fn write(&self, w: &mut WireWriter) {
        w.bool(self.success);
        w.optional(self.code.as_ref(), |w, &v| w.int(v));
        w.optional(self.status.as_ref(), |w, &v| w.int(v));
        w.optional(self.key.as_deref(), |w, s| w.string(s));
        w.optional(self.url.as_deref(), |w, s| w.string(s));
        w.optional(self.imageBase64.as_deref(), |w, s| w.string(s));
        w.optional(self.cookie.as_deref(), |w, s| w.string(s));
        w.optional(self.verifyUrl.as_deref(), |w, s| w.string(s));
        w.bool(self.challenge);
        w.optional(self.message.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            success: r.bool()?,
            code: r.optional(|r| r.int())?,
            status: r.optional(|r| r.int())?,
            key: r.optional(|r| r.string())?,
            url: r.optional(|r| r.string())?,
            imageBase64: r.optional(|r| r.string())?,
            cookie: r.optional(|r| r.string())?,
            verifyUrl: r.optional(|r| r.string())?,
            challenge: r.bool()?,
            message: r.optional(|r| r.string())?,
        })
    }
}

/// Plain-text search suggestions.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SuggestionPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<String>,
}

impl SuggestionPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| r.string())?,
        })
    }
}

/// At least one primary lyric format, with optional translation and romanization.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Lyrics {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Primary LRC lyric text.
    pub lrc: Option<String>,
    /// Primary TTML lyric text.
    pub ttml: Option<String>,
    /// Primary YRC lyric text.
    pub yrc: Option<String>,
    /// Translated lyric text when available.
    pub translation: Option<String>,
    /// Romanized lyric text when available.
    pub romanization: Option<String>,
}

impl Lyrics {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.optional(self.lrc.as_deref(), |w, s| w.string(s));
        w.optional(self.ttml.as_deref(), |w, s| w.string(s));
        w.optional(self.yrc.as_deref(), |w, s| w.string(s));
        w.optional(self.translation.as_deref(), |w, s| w.string(s));
        w.optional(self.romanization.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            lrc: r.optional(|r| r.string())?,
            ttml: r.optional(|r| r.string())?,
            yrc: r.optional(|r| r.string())?,
            translation: r.optional(|r| r.string())?,
            romanization: r.optional(|r| r.string())?,
        })
    }
}

/// Playlist metadata, separate from paginated track contents.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Playlist {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Optional cover image URL.
    pub coverUrl: Option<String>,
    /// Known track count; null means unknown where nullable.
    pub trackCount: Option<i32>,
    /// Opaque creator identity.
    pub creatorId: Option<String>,
    /// Creator display name.
    pub creatorName: Option<String>,
    /// Descriptive text; null means unavailable or unchanged according to the record.
    pub description: Option<String>,
}

impl Playlist {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.title);
        w.optional(self.coverUrl.as_deref(), |w, s| w.string(s));
        w.optional(self.trackCount.as_ref(), |w, &v| w.int(v));
        w.optional(self.creatorId.as_deref(), |w, s| w.string(s));
        w.optional(self.creatorName.as_deref(), |w, s| w.string(s));
        w.optional(self.description.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            title: r.string()?,
            coverUrl: r.optional(|r| r.string())?,
            trackCount: r.optional(|r| r.int())?,
            creatorId: r.optional(|r| r.string())?,
            creatorName: r.optional(|r| r.string())?,
            description: r.optional(|r| r.string())?,
        })
    }
}

/// A page of playlists with optional pagination metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Playlist>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
}

impl PlaylistPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Playlist::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
        })
    }
}

/// Album metadata with an optional release timestamp.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Album {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Ordered credited artists.
    pub artists: Vec<Artist>,
    /// Optional cover image URL.
    pub coverUrl: Option<String>,
    /// Known track count; null means unknown where nullable.
    pub trackCount: Option<i32>,
    /// Release time in Unix epoch milliseconds; null means unknown.
    pub releasedAtEpochMs: Option<i64>,
    /// Release company when known.
    pub company: Option<String>,
    /// Descriptive text; null means unavailable or unchanged according to the record.
    pub description: Option<String>,
}

impl Album {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.title);
        w.list(&self.artists, |w, item| item.write(w));
        w.optional(self.coverUrl.as_deref(), |w, s| w.string(s));
        w.optional(self.trackCount.as_ref(), |w, &v| w.int(v));
        w.optional(self.releasedAtEpochMs.as_ref(), |w, &v| w.long(v));
        w.optional(self.company.as_deref(), |w, s| w.string(s));
        w.optional(self.description.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            title: r.string()?,
            artists: r.list(|r| Artist::read(r))?,
            coverUrl: r.optional(|r| r.string())?,
            trackCount: r.optional(|r| r.int())?,
            releasedAtEpochMs: r.optional(|r| r.long())?,
            company: r.optional(|r| r.string())?,
            description: r.optional(|r| r.string())?,
        })
    }
}

/// A page/selection of albums with optional pagination metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AlbumPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Album>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
}

impl AlbumPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Album::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
        })
    }
}

/// Album metadata and complete bounded track contents.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AlbumDetails {
    /// Album name.
    pub album: Album,
    /// Ordered tracks.
    pub tracks: Vec<Track>,
}

impl AlbumDetails {
    pub fn write(&self, w: &mut WireWriter) {
        self.album.write(w);
        w.list(&self.tracks, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            album: Album::read(r)?,
            tracks: r.list(|r| Track::read(r))?,
        })
    }
}

/// Artist metadata with optional known catalog counts.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CatalogArtist {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional avatar image URL.
    pub avatarUrl: Option<String>,
    /// Descriptive text; null means unavailable or unchanged according to the record.
    pub description: Option<String>,
    /// Alternative display names.
    pub aliases: Vec<String>,
    /// Known album count; null means unknown.
    pub albumCount: Option<i32>,
    /// Known track count; null means unknown where nullable.
    pub trackCount: Option<i32>,
}

impl CatalogArtist {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.name);
        w.optional(self.avatarUrl.as_deref(), |w, s| w.string(s));
        w.optional(self.description.as_deref(), |w, s| w.string(s));
        w.list(&self.aliases, |w, s| w.string(s));
        w.optional(self.albumCount.as_ref(), |w, &v| w.int(v));
        w.optional(self.trackCount.as_ref(), |w, &v| w.int(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            name: r.string()?,
            avatarUrl: r.optional(|r| r.string())?,
            description: r.optional(|r| r.string())?,
            aliases: r.list(|r| r.string())?,
            albumCount: r.optional(|r| r.int())?,
            trackCount: r.optional(|r| r.int())?,
        })
    }
}

/// A page of artist-shaped profiles with optional pagination metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ArtistPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<CatalogArtist>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
}

impl ArtistPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| CatalogArtist::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
        })
    }
}

/// Discovery banner with provider-specific target type and opaque target ID.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Banner {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Image URL.
    pub imageUrl: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Provider-specific banner target discriminator.
    pub targetType: i32,
    /// Opaque target resource identity.
    pub targetId: String,
}

impl Banner {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.imageUrl);
        w.string(&self.title);
        w.int(self.targetType);
        w.string(&self.targetId);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            imageUrl: r.string()?,
            title: r.string()?,
            targetType: r.int()?,
            targetId: r.string()?,
        })
    }
}

/// A bounded list of discovery banners.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BannerPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Banner>,
}

impl BannerPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Banner::read(r))?,
        })
    }
}

/// Ranking-list metadata; its ID is used to address the corresponding catalog resource.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Ranking {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Optional cover image URL.
    pub coverUrl: Option<String>,
    /// Human-readable ranking update frequency.
    pub updateFrequency: Option<String>,
    /// Known track count; null means unknown where nullable.
    pub trackCount: i32,
}

impl Ranking {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.title);
        w.optional(self.coverUrl.as_deref(), |w, s| w.string(s));
        w.optional(self.updateFrequency.as_deref(), |w, s| w.string(s));
        w.int(self.trackCount);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            title: r.string()?,
            coverUrl: r.optional(|r| r.string())?,
            updateFrequency: r.optional(|r| r.string())?,
            trackCount: r.int()?,
        })
    }
}

/// A bounded list of ranking-list metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RankingPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Ranking>,
}

impl RankingPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Ranking::read(r))?,
        })
    }
}

/// Item-offset pagination; offset counts items, not pages.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PageRequest {
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl PageRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// Opaque-cursor pagination; null and the empty string remain distinct.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CursorRequest {
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl CursorRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.int(self.limit);
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            limit: r.int()?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// Comment deletion scoped to a resource and resource type.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DeleteCommentRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Operation-specific discriminator; see the operation and record-specific constraints.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Existing v1 numeric comment identity; must stay within JavaScript safe-integer bounds.
    pub commentId: i64,
}

impl DeleteCommentRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.r#type);
        w.long(self.commentId);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            r#type: r.string()?,
            commentId: r.long()?,
        })
    }
}

/// Social user metadata with an opaque provider-local identity.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SocialUser {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Public display nickname.
    pub nickname: String,
    /// Optional avatar image URL.
    pub avatarUrl: Option<String>,
}

impl SocialUser {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.nickname);
        w.optional(self.avatarUrl.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            nickname: r.string()?,
            avatarUrl: r.optional(|r| r.string())?,
        })
    }
}

/// A page of social users with optional pagination metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SocialUserPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<SocialUser>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
}

impl SocialUserPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| SocialUser::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
        })
    }
}

/// Separate unread counters for each notification/message family.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UnreadCount {
    /// Unread private-message count.
    pub privateMessages: i32,
    /// Ordered comments bounded by the request limit.
    pub comments: i32,
    /// Unread mention count.
    pub mentions: i32,
    /// Unread system-notice count.
    pub notices: i32,
}

impl UnreadCount {
    pub fn write(&self, w: &mut WireWriter) {
        w.int(self.privateMessages);
        w.int(self.comments);
        w.int(self.mentions);
        w.int(self.notices);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            privateMessages: r.int()?,
            comments: r.int()?,
            mentions: r.int()?,
            notices: r.int()?,
        })
    }
}

/// One plain-text private message with sender, recipient and Unix timestamp.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PrivateMessage {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Message/notification sender profile.
    pub sender: SocialUser,
    /// Message recipient profile.
    pub recipient: SocialUser,
    /// Plain text, never an upstream JSON message envelope.
    pub text: String,
    /// Send time in Unix epoch milliseconds.
    pub sentAtEpochMs: i64,
}

impl PrivateMessage {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        self.sender.write(w);
        self.recipient.write(w);
        w.string(&self.text);
        w.long(self.sentAtEpochMs);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            sender: SocialUser::read(r)?,
            recipient: SocialUser::read(r)?,
            text: r.string()?,
            sentAtEpochMs: r.long()?,
        })
    }
}

/// Newest-first private messages with optional continuation metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MessagePage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<PrivateMessage>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl MessagePage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| PrivateMessage::read(r))?,
            hasMore: r.optional(|r| r.bool())?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// One contact's conversation summary and unread count.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Conversation {
    /// User attribution.
    pub user: SocialUser,
    /// Latest known message, or null.
    pub lastMessage: Option<PrivateMessage>,
    /// Known unread-message count.
    pub unreadCount: i32,
}

impl Conversation {
    pub fn write(&self, w: &mut WireWriter) {
        self.user.write(w);
        w.optional(self.lastMessage.as_ref(), |w, item| item.write(w));
        w.int(self.unreadCount);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            user: SocialUser::read(r)?,
            lastMessage: r.optional(|r| PrivateMessage::read(r))?,
            unreadCount: r.int()?,
        })
    }
}

/// Conversation summaries with optional continuation metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ConversationPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Conversation>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl ConversationPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Conversation::read(r))?,
            hasMore: r.optional(|r| r.bool())?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// Private-message history input for one user and an opaque cursor.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MessageHistoryRequest {
    /// Opaque provider-local user identity, not the host account-slot key.
    pub userId: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl MessageHistoryRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.userId);
        w.int(self.limit);
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            userId: r.string()?,
            limit: r.int()?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// Plain-text message input for one or more recipients.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SendMessageRequest {
    /// Recipient provider-local user IDs, 1..100 where used by message requests.
    pub userIds: Vec<String>,
    /// Plain text, never an upstream JSON message envelope.
    pub text: String,
}

impl SendMessageRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.userIds, |w, s| w.string(s));
        w.string(&self.text);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            userIds: r.list(|r| r.string())?,
            text: r.string()?,
        })
    }
}

/// A listen-together room invitation and inviter identity.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenInvitationRequest {
    /// Opaque listen-together room identity.
    pub roomId: String,
    /// Opaque inviting-user identity.
    pub inviterId: String,
}

impl ListenInvitationRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.roomId);
        w.string(&self.inviterId);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            roomId: r.string()?,
            inviterId: r.string()?,
        })
    }
}

/// An opaque listen-together room identity.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenRoomRequest {
    /// Opaque listen-together room identity.
    pub roomId: String,
}

impl ListenRoomRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.roomId);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            roomId: r.string()?,
        })
    }
}

/// A room member and current online state.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenMember {
    /// User attribution.
    pub user: SocialUser,
    /// Whether the member is currently online.
    pub online: bool,
}

impl ListenMember {
    pub fn write(&self, w: &mut WireWriter) {
        self.user.write(w);
        w.bool(self.online);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            user: SocialUser::read(r)?,
            online: r.bool()?,
        })
    }
}

/// Listen-together room metadata with optional timing information in milliseconds.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenRoom {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Current room members.
    pub members: Vec<ListenMember>,
    /// Opaque creator identity.
    pub creatorId: Option<String>,
    /// Provider-defined room type.
    pub roomType: Option<String>,
    /// Creation time in Unix epoch milliseconds.
    pub createdAtEpochMs: Option<i64>,
    /// Effective listening duration in milliseconds.
    pub effectiveDurationMs: Option<i64>,
    /// Wait interval in milliseconds.
    pub waitMs: Option<i64>,
}

impl ListenRoom {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.list(&self.members, |w, item| item.write(w));
        w.optional(self.creatorId.as_deref(), |w, s| w.string(s));
        w.optional(self.roomType.as_deref(), |w, s| w.string(s));
        w.optional(self.createdAtEpochMs.as_ref(), |w, &v| w.long(v));
        w.optional(self.effectiveDurationMs.as_ref(), |w, &v| w.long(v));
        w.optional(self.waitMs.as_ref(), |w, &v| w.long(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            members: r.list(|r| ListenMember::read(r))?,
            creatorId: r.optional(|r| r.string())?,
            roomType: r.optional(|r| r.string())?,
            createdAtEpochMs: r.optional(|r| r.long())?,
            effectiveDurationMs: r.optional(|r| r.long())?,
            waitMs: r.optional(|r| r.long())?,
        })
    }
}

/// Current membership; room is required when inRoom is true.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenStatus {
    /// Whether the current account belongs to a room; true requires room metadata.
    pub inRoom: bool,
    /// Current room metadata.
    pub room: Option<ListenRoom>,
    /// Provider-defined connection state.
    pub connectionStatus: Option<String>,
}

impl ListenStatus {
    pub fn write(&self, w: &mut WireWriter) {
        w.bool(self.inRoom);
        w.optional(self.room.as_ref(), |w, item| item.write(w));
        w.optional(self.connectionStatus.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            inRoom: r.bool()?,
            room: r.optional(|r| ListenRoom::read(r))?,
            connectionStatus: r.optional(|r| r.string())?,
        })
    }
}

/// Invitation check result without side effects.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenRoomCheck {
    /// Whether the invitation can be accepted.
    pub joinable: bool,
    /// Sanitized human-readable explanation, not a credential or upstream response dump.
    pub message: Option<String>,
}

impl ListenRoomCheck {
    pub fn write(&self, w: &mut WireWriter) {
        w.bool(self.joinable);
        w.optional(self.message.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            joinable: r.bool()?,
            message: r.optional(|r| r.string())?,
        })
    }
}

/// Current room playback state and millisecond position.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenHeartbeatRequest {
    /// Opaque listen-together room identity.
    pub roomId: String,
    /// Opaque audio-track identity.
    pub trackId: Option<String>,
    /// Playback position in milliseconds.
    pub positionMs: i64,
    /// PLAY or PAUSE.
    pub playStatus: String,
}

impl ListenHeartbeatRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.roomId);
        w.optional(self.trackId.as_deref(), |w, s| w.string(s));
        w.long(self.positionMs);
        w.string(&self.playStatus);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            roomId: r.string()?,
            trackId: r.optional(|r| r.string())?,
            positionMs: r.long()?,
            playStatus: r.string()?,
        })
    }
}

/// Positive next heartbeat interval in milliseconds.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenHeartbeat {
    /// Positive next heartbeat interval in milliseconds, not seconds.
    pub intervalMs: i64,
}

impl ListenHeartbeat {
    pub fn write(&self, w: &mut WireWriter) {
        w.long(self.intervalMs);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            intervalMs: r.long()?,
        })
    }
}

/// Sequenced room playback command; GOTO requires a target track ID.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenCommand {
    /// PLAY, PAUSE, PROGRESS or GOTO; GOTO requires trackId.
    pub action: String,
    /// Opaque audio-track identity.
    pub trackId: Option<String>,
    /// Playback position in milliseconds.
    pub positionMs: i64,
    /// Nonnegative client command sequence within safe-integer bounds.
    pub sequence: i64,
    /// PLAY or PAUSE.
    pub playStatus: String,
    /// Previous track identity when known.
    pub formerTrackId: Option<String>,
    /// Nonnegative server command sequence within safe-integer bounds.
    pub serverSequence: i64,
    /// Opaque provider-local user identity, not the host account-slot key.
    pub userId: Option<String>,
}

impl ListenCommand {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.action);
        w.optional(self.trackId.as_deref(), |w, s| w.string(s));
        w.long(self.positionMs);
        w.long(self.sequence);
        w.string(&self.playStatus);
        w.optional(self.formerTrackId.as_deref(), |w, s| w.string(s));
        w.long(self.serverSequence);
        w.optional(self.userId.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            action: r.string()?,
            trackId: r.optional(|r| r.string())?,
            positionMs: r.long()?,
            sequence: r.long()?,
            playStatus: r.string()?,
            formerTrackId: r.optional(|r| r.string())?,
            serverSequence: r.long()?,
            userId: r.optional(|r| r.string())?,
        })
    }
}

/// A playback command scoped to a room.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenCommandRequest {
    /// Opaque listen-together room identity.
    pub roomId: String,
    /// Sequenced playback command.
    pub command: ListenCommand,
}

impl ListenCommandRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.roomId);
        self.command.write(w);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            roomId: r.string()?,
            command: ListenCommand::read(r)?,
        })
    }
}

/// Ordered queue synchronization with explicit action and version.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenQueueRequest {
    /// Opaque listen-together room identity.
    pub roomId: String,
    /// Ordered queue IDs; duplicates are meaningful. Empty REPLACE clears the queue.
    pub trackIds: Vec<String>,
    /// Nonnegative queue version within safe-integer bounds.
    pub version: i64,
    /// Opaque provider-local user identity, not the host account-slot key.
    pub userId: String,
    /// REPLACE, ADD or DELETE. Empty REPLACE explicitly clears the queue.
    pub action: String,
}

impl ListenQueueRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.roomId);
        w.list(&self.trackIds, |w, s| w.string(s));
        w.long(self.version);
        w.string(&self.userId);
        w.string(&self.action);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            roomId: r.string()?,
            trackIds: r.list(|r| r.string())?,
            version: r.long()?,
            userId: r.string()?,
            action: r.string()?,
        })
    }
}

/// Per-user queue version.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenQueueVersion {
    /// Opaque provider-local user identity, not the host account-slot key.
    pub userId: String,
    /// Nonnegative queue version within safe-integer bounds.
    pub version: i64,
}

impl ListenQueueVersion {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.userId);
        w.long(self.version);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            userId: r.string()?,
            version: r.long()?,
        })
    }
}

/// An explicit queue snapshot, including a meaningful empty queue.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenPlaylist {
    /// Explicit ordered queue contents, including a meaningful empty queue.
    pub trackIds: Vec<String>,
    /// True replaces the queue; false applies the provider's incremental snapshot semantics.
    pub replace: bool,
    /// Per-user queue versions.
    pub versions: Vec<ListenQueueVersion>,
}

impl ListenPlaylist {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.trackIds, |w, s| w.string(s));
        w.bool(self.replace);
        w.list(&self.versions, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            trackIds: r.list(|r| r.string())?,
            replace: r.bool()?,
            versions: r.list(|r| ListenQueueVersion::read(r))?,
        })
    }
}

/// Optional pending command and queue; null queue means no snapshot, not clear.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ListenSnapshot {
    /// Sequenced playback command.
    pub command: Option<ListenCommand>,
    /// Queue snapshot; null means no snapshot, not an empty queue.
    pub playlist: Option<ListenPlaylist>,
}

impl ListenSnapshot {
    pub fn write(&self, w: &mut WireWriter) {
        w.optional(self.command.as_ref(), |w, item| item.write(w));
        w.optional(self.playlist.as_ref(), |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            command: r.optional(|r| ListenCommand::read(r))?,
            playlist: r.optional(|r| ListenPlaylist::read(r))?,
        })
    }
}

/// MV/video metadata. The operation, not ID syntax, determines the resource type.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct VideoItem {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Optional cover image URL.
    pub coverUrl: Option<String>,
    /// Duration in milliseconds; never infer seconds from magnitude.
    pub durationMs: i64,
    /// Ordered credited creators.
    pub creators: Vec<Artist>,
    /// Descriptive text; null means unavailable or unchanged according to the record.
    pub description: Option<String>,
    /// Known play count; null means unknown.
    pub playCount: Option<i64>,
}

impl VideoItem {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.title);
        w.optional(self.coverUrl.as_deref(), |w, s| w.string(s));
        w.long(self.durationMs);
        w.list(&self.creators, |w, item| item.write(w));
        w.optional(self.description.as_deref(), |w, s| w.string(s));
        w.optional(self.playCount.as_ref(), |w, &v| w.long(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            title: r.string()?,
            coverUrl: r.optional(|r| r.string())?,
            durationMs: r.long()?,
            creators: r.list(|r| Artist::read(r))?,
            description: r.optional(|r| r.string())?,
            playCount: r.optional(|r| r.long())?,
        })
    }
}

/// A page of MVs or videos with optional pagination metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct VideoPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<VideoItem>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
}

impl VideoPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| VideoItem::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
        })
    }
}

/// MV area selector and item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct VideoBrowseRequest {
    /// Provider-supported area selector; do not silently reinterpret unknown values.
    pub area: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl VideoBrowseRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.area);
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            area: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// MV/video identity and requested vertical resolution.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct VideoResolveRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Requested vertical resolution in pixels; actual output may differ.
    pub resolution: i32,
}

impl VideoResolveRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.resolution);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            resolution: r.int()?,
        })
    }
}

/// Explicit desired subscription state for the operation's resource type.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Explicit desired subscription state, not a toggle.
    pub subscribed: bool,
}

impl SubscriptionRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.bool(self.subscribed);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            subscribed: r.bool()?,
        })
    }
}

/// Opaque category/group/tag identity and display name.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Category {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display name.
    pub name: String,
}

impl Category {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.name);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            name: r.string()?,
        })
    }
}

/// A bounded list of categories, groups or tags.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CategoryPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Category>,
}

impl CategoryPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Category::read(r))?,
        })
    }
}

/// DJ-radio metadata, distinct from an individual program.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DjRadio {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Optional cover image URL.
    pub coverUrl: Option<String>,
    /// Descriptive text; null means unavailable or unchanged according to the record.
    pub description: Option<String>,
    /// Creator profile when known.
    pub creator: Option<SocialUser>,
    /// Known program count; null means unknown.
    pub programCount: Option<i32>,
}

impl DjRadio {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.title);
        w.optional(self.coverUrl.as_deref(), |w, s| w.string(s));
        w.optional(self.description.as_deref(), |w, s| w.string(s));
        w.optional(self.creator.as_ref(), |w, item| item.write(w));
        w.optional(self.programCount.as_ref(), |w, &v| w.int(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            title: r.string()?,
            coverUrl: r.optional(|r| r.string())?,
            description: r.optional(|r| r.string())?,
            creator: r.optional(|r| SocialUser::read(r))?,
            programCount: r.optional(|r| r.int())?,
        })
    }
}

/// A page of DJ radios.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DjRadioPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<DjRadio>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
}

impl DjRadioPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| DjRadio::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
        })
    }
}

/// An individual DJ program and its playable audio track.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DjProgram {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Opaque DJ-radio identity.
    pub radioId: Option<String>,
    /// Audio track metadata.
    pub track: Track,
    /// Descriptive text; null means unavailable or unchanged according to the record.
    pub description: Option<String>,
    /// Publication time in Unix epoch milliseconds.
    pub publishedAtEpochMs: Option<i64>,
}

impl DjProgram {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.title);
        w.optional(self.radioId.as_deref(), |w, s| w.string(s));
        self.track.write(w);
        w.optional(self.description.as_deref(), |w, s| w.string(s));
        w.optional(self.publishedAtEpochMs.as_ref(), |w, &v| w.long(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            title: r.string()?,
            radioId: r.optional(|r| r.string())?,
            track: Track::read(r)?,
            description: r.optional(|r| r.string())?,
            publishedAtEpochMs: r.optional(|r| r.long())?,
        })
    }
}

/// A page of DJ programs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DjProgramPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<DjProgram>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
}

impl DjProgramPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| DjProgram::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
        })
    }
}

/// DJ-radio identity, program ordering and item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DjProgramsRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
    /// True requests ascending program order; false requests descending order.
    pub ascending: bool,
}

impl DjProgramsRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.limit);
        w.int(self.offset);
        w.bool(self.ascending);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
            ascending: r.bool()?,
        })
    }
}

/// Optional calendar range boundaries in Unix epoch milliseconds.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CalendarRequest {
    /// Optional inclusive range start in Unix epoch milliseconds; upstream range semantics belong to the provider.
    pub startAtEpochMs: Option<i64>,
    /// Optional range end in Unix epoch milliseconds; upstream range semantics belong to the provider.
    pub endAtEpochMs: Option<i64>,
}

impl CalendarRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.optional(self.startAtEpochMs.as_ref(), |w, &v| w.long(v));
        w.optional(self.endAtEpochMs.as_ref(), |w, &v| w.long(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            startAtEpochMs: r.optional(|r| r.long())?,
            endAtEpochMs: r.optional(|r| r.long())?,
        })
    }
}

/// Calendar entry with a timestamp and optional associated resource.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CalendarEntry {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Start time in Unix epoch milliseconds.
    pub startsAtEpochMs: i64,
    /// End time in Unix epoch milliseconds, no earlier than the start when known.
    pub endsAtEpochMs: Option<i64>,
    /// Optional cover image URL.
    pub coverUrl: Option<String>,
    /// Opaque associated resource identity.
    pub resourceId: Option<String>,
    /// Associated resource kind, not an endpoint path.
    pub resourceType: Option<String>,
}

impl CalendarEntry {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.title);
        w.long(self.startsAtEpochMs);
        w.optional(self.endsAtEpochMs.as_ref(), |w, &v| w.long(v));
        w.optional(self.coverUrl.as_deref(), |w, s| w.string(s));
        w.optional(self.resourceId.as_deref(), |w, s| w.string(s));
        w.optional(self.resourceType.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            title: r.string()?,
            startsAtEpochMs: r.long()?,
            endsAtEpochMs: r.optional(|r| r.long())?,
            coverUrl: r.optional(|r| r.string())?,
            resourceId: r.optional(|r| r.string())?,
            resourceType: r.optional(|r| r.string())?,
        })
    }
}

/// A bounded list of calendar entries.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CalendarPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<CalendarEntry>,
}

impl CalendarPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| CalendarEntry::read(r))?,
        })
    }
}

/// Social-feed entry with plain text and typed resource/image references.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SocialEvent {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// User attribution.
    pub user: SocialUser,
    /// Plain text, never an upstream JSON message envelope.
    pub text: String,
    /// Publication time in Unix epoch milliseconds.
    pub publishedAtEpochMs: i64,
    /// Opaque associated resource identity.
    pub resourceId: Option<String>,
    /// Associated resource kind, not an endpoint path.
    pub resourceType: Option<String>,
    /// Ordered image URLs.
    pub imageUrls: Vec<String>,
}

impl SocialEvent {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        self.user.write(w);
        w.string(&self.text);
        w.long(self.publishedAtEpochMs);
        w.optional(self.resourceId.as_deref(), |w, s| w.string(s));
        w.optional(self.resourceType.as_deref(), |w, s| w.string(s));
        w.list(&self.imageUrls, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            user: SocialUser::read(r)?,
            text: r.string()?,
            publishedAtEpochMs: r.long()?,
            resourceId: r.optional(|r| r.string())?,
            resourceType: r.optional(|r| r.string())?,
            imageUrls: r.list(|r| r.string())?,
        })
    }
}

/// Social events with opaque continuation metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EventPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<SocialEvent>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl EventPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| SocialEvent::read(r))?,
            hasMore: r.optional(|r| r.bool())?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// Current-account social-feed cursor input.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EventPageRequest {
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl EventPageRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.int(self.limit);
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            limit: r.int()?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// User-scoped social-feed cursor input.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserEventRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl UserEventRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.limit);
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            limit: r.int()?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// Social-event forwarding input with plain text.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ForwardEventRequest {
    /// Opaque social-event identity.
    pub eventId: String,
    /// Opaque provider-local user identity, not the host account-slot key.
    pub userId: String,
    /// Plain text, never an upstream JSON message envelope.
    pub text: String,
}

impl ForwardEventRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.eventId);
        w.string(&self.userId);
        w.string(&self.text);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            eventId: r.string()?,
            userId: r.string()?,
            text: r.string()?,
        })
    }
}

/// Typed resource sharing input with accompanying plain text.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ShareResourceRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Operation-specific discriminator; see the operation and record-specific constraints.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Plain text, never an upstream JSON message envelope.
    pub text: String,
}

impl ShareResourceRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.r#type);
        w.string(&self.text);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            r#type: r.string()?,
            text: r.string()?,
        })
    }
}

/// Partial playlist metadata update; null leaves fields unchanged.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistUpdateRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Display name.
    pub name: Option<String>,
    /// Descriptive text; null means unavailable or unchanged according to the record.
    pub description: Option<String>,
    /// Null leaves tags unchanged; empty clears them; at most 20 nonblank tags of at most 128 characters.
    pub tags: Option<Vec<String>>,
}

impl PlaylistUpdateRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.optional(self.name.as_deref(), |w, s| w.string(s));
        w.optional(self.description.as_deref(), |w, s| w.string(s));
        w.optional(self.tags.as_deref(), |w, list| w.list(list, |w, s| w.string(s)));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            name: r.optional(|r| r.string())?,
            description: r.optional(|r| r.string())?,
            tags: r.optional(|r| r.list(|r| r.string()))?,
        })
    }
}

/// Partial profile update; at least one non-null field is required.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserProfileUpdateRequest {
    /// Public display nickname.
    pub nickname: Option<String>,
    /// Profile signature text; null means unchanged for patch requests.
    pub signature: Option<String>,
    /// Existing profile selector: 0, 1 or 2.
    pub gender: Option<i32>,
    /// Birthday timestamp in Unix epoch milliseconds; negative pre-1970 values are allowed.
    pub birthdayEpochMs: Option<i64>,
    /// Nonnegative provider-defined province code.
    pub province: Option<i32>,
    /// Nonnegative provider-defined city code.
    pub city: Option<i32>,
}

impl UserProfileUpdateRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.optional(self.nickname.as_deref(), |w, s| w.string(s));
        w.optional(self.signature.as_deref(), |w, s| w.string(s));
        w.optional(self.gender.as_ref(), |w, &v| w.int(v));
        w.optional(self.birthdayEpochMs.as_ref(), |w, &v| w.long(v));
        w.optional(self.province.as_ref(), |w, &v| w.int(v));
        w.optional(self.city.as_ref(), |w, &v| w.int(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            nickname: r.optional(|r| r.string())?,
            signature: r.optional(|r| r.string())?,
            gender: r.optional(|r| r.int())?,
            birthdayEpochMs: r.optional(|r| r.long())?,
            province: r.optional(|r| r.int())?,
            city: r.optional(|r| r.int())?,
        })
    }
}

/// Daily sign-in acknowledgement with optional awarded points.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SigninResult {
    /// Explicit mutation/authentication acknowledgement; false is not an empty successful data page.
    pub success: bool,
    /// Awarded points when known.
    pub points: Option<i32>,
}

impl SigninResult {
    pub fn write(&self, w: &mut WireWriter) {
        w.bool(self.success);
        w.optional(self.points.as_ref(), |w, &v| w.int(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            success: r.bool()?,
            points: r.optional(|r| r.int())?,
        })
    }
}

/// Explicit desired follow state for a social user.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FollowUserRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Explicit desired follow state, not a toggle.
    pub followed: bool,
}

impl FollowUserRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.bool(self.followed);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            followed: r.bool()?,
        })
    }
}

/// Cloud import identity and catalog-match identity.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudImportRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Opaque upstream catalog-match identity.
    pub matchId: String,
}

impl CloudImportRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.matchId);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            matchId: r.string()?,
        })
    }
}

/// Cloud item and corrected catalog association for a user.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudMatchRequest {
    /// Opaque provider-local user identity, not the host account-slot key.
    pub userId: String,
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Opaque corrected catalog identity.
    pub adjustId: String,
}

impl CloudMatchRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.userId);
        w.string(&self.id);
        w.string(&self.adjustId);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            userId: r.string()?,
            id: r.string()?,
            adjustId: r.string()?,
        })
    }
}

/// Artist discovery filters and item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ArtistBrowseRequest {
    /// Artist type selector: -1, 1, 2 or 3.
    #[serde(rename = "type")]
    pub r#type: i32,
    /// Provider-supported numeric area selector; the SDK default is -100.
    pub area: i32,
    /// Artist initial selector; at most 16 characters.
    pub initial: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl ArtistBrowseRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.int(self.r#type);
        w.int(self.area);
        w.string(&self.initial);
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            r#type: r.int()?,
            area: r.int()?,
            initial: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// Known follower count.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FollowCount {
    /// Known count.
    pub count: i64,
}

impl FollowCount {
    pub fn write(&self, w: &mut WireWriter) {
        w.long(self.count);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            count: r.long()?,
        })
    }
}

/// Dates available for historical recommendation queries.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RecommendationHistory {
    /// Available recommendation-history dates.
    pub dates: Vec<String>,
}

impl RecommendationHistory {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.dates, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            dates: r.list(|r| r.string())?,
        })
    }
}

/// One recommendation-history date in YYYY-MM-DD syntax.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HistoryTracksRequest {
    /// YYYY-MM-DD syntax for recommendation-history lookup.
    pub date: String,
}

impl HistoryTracksRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.date);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            date: r.string()?,
        })
    }
}

/// Audio availability check at an explicit bitrate.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrackAvailabilityRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Bitrate in bits per second, not kilobits per second.
    pub bitrate: i32,
}

impl TrackAvailabilityRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.bitrate);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            bitrate: r.int()?,
        })
    }
}

/// Availability result with optional sanitized explanation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrackAvailability {
    /// Whether this resource/quality is currently available to the account.
    pub available: bool,
    /// Sanitized human-readable explanation, not a credential or upstream response dump.
    pub message: Option<String>,
}

impl TrackAvailability {
    pub fn write(&self, w: &mut WireWriter) {
        w.bool(self.available);
        w.optional(self.message.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            available: r.bool()?,
            message: r.optional(|r| r.string())?,
        })
    }
}

/// Additive cursor-aware comment input preserving the old request layout.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommentQuery {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// music, song, mv, playlist, album, dj, video or event; the high-level namespaced helper uses music for song IDs.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
    /// 1 = recommended, 2 = hot, 3 = latest. Responses carry the effective server sort.
    pub sortType: i32,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl CommentQuery {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.r#type);
        w.int(self.limit);
        w.int(self.offset);
        w.int(self.sortType);
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            r#type: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
            sortType: r.int()?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// Area-filtered new-album browsing with item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AlbumPageRequest {
    /// Provider-supported area selector; do not silently reinterpret unknown values.
    pub area: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl AlbumPageRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.area);
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            area: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// Album chart kind, area, optional year/month and item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AlbumChartRequest {
    /// Provider-supported area selector; do not silently reinterpret unknown values.
    pub area: String,
    /// Ranking/chart selector: new or hot.
    pub kind: String,
    /// Optional chart year, 1..9999.
    pub year: Option<i32>,
    /// Optional chart month, 1..12.
    pub month: Option<i32>,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl AlbumChartRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.area);
        w.string(&self.kind);
        w.optional(self.year.as_ref(), |w, &v| w.int(v));
        w.optional(self.month.as_ref(), |w, &v| w.int(v));
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            area: r.string()?,
            kind: r.string()?,
            year: r.optional(|r| r.int())?,
            month: r.optional(|r| r.int())?,
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// User-scoped opaque-cursor pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserCursorRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl UserCursorRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.int(self.limit);
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            limit: r.int()?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// Plain-text notification with optional sender and typed resource attribution.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Notification {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Plain text, never an upstream JSON message envelope.
    pub text: String,
    /// Send time in Unix epoch milliseconds.
    pub sentAtEpochMs: i64,
    /// Message/notification sender profile.
    pub sender: Option<SocialUser>,
    /// Opaque associated resource identity.
    pub resourceId: Option<String>,
    /// Associated resource kind, not an endpoint path.
    pub resourceType: Option<String>,
    /// Existing v1 numeric comment identity; must stay within JavaScript safe-integer bounds.
    pub commentId: Option<i64>,
}

impl Notification {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.text);
        w.long(self.sentAtEpochMs);
        w.optional(self.sender.as_ref(), |w, item| item.write(w));
        w.optional(self.resourceId.as_deref(), |w, s| w.string(s));
        w.optional(self.resourceType.as_deref(), |w, s| w.string(s));
        w.optional(self.commentId.as_ref(), |w, &v| w.long(v));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            text: r.string()?,
            sentAtEpochMs: r.long()?,
            sender: r.optional(|r| SocialUser::read(r))?,
            resourceId: r.optional(|r| r.string())?,
            resourceType: r.optional(|r| r.string())?,
            commentId: r.optional(|r| r.long())?,
        })
    }
}

/// Notification items with optional total and continuation metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotificationPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<Notification>,
    /// Known total count, not current-page length; null means unknown.
    pub total: Option<i64>,
    /// Known continuation state; null means unknown, not false.
    pub hasMore: Option<bool>,
    /// Opaque continuation token. Preserve empty strings and punctuation; at most 4096 characters for extended operations.
    pub cursor: Option<String>,
}

impl NotificationPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
        w.optional(self.total.as_ref(), |w, &v| w.long(v));
        w.optional(self.hasMore.as_ref(), |w, &v| w.bool(v));
        w.optional(self.cursor.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| Notification::read(r))?,
            total: r.optional(|r| r.long())?,
            hasMore: r.optional(|r| r.bool())?,
            cursor: r.optional(|r| r.string())?,
        })
    }
}

/// Nonempty unique playlist IDs in the desired order.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistOrderRequest {
    /// 1..500 unique IDs in the desired order; duplicates are rejected, not deduplicated.
    pub playlistIds: Vec<String>,
}

impl PlaylistOrderRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.playlistIds, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            playlistIds: r.list(|r| r.string())?,
        })
    }
}

/// Playlist identity and nonempty unique track IDs in the desired order.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistTrackOrderRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// 1..500 unique IDs in the desired order; duplicates are rejected.
    pub trackIds: Vec<String>,
}

impl PlaylistTrackOrderRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.list(&self.trackIds, |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            trackIds: r.list(|r| r.string())?,
        })
    }
}

/// Playlist identity and an already uploaded image ID.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistCoverRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Opaque uploaded-image identity, not a file path.
    pub imageId: String,
}

impl PlaylistCoverRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.imageId);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            imageId: r.string()?,
        })
    }
}

/// A basename, supported image MIME type and at most 1 MiB of nonempty binary content.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ImageUploadRequest {
    /// Basename only, 1..256 characters; no path separators, reserved filename punctuation or controls.
    pub filename: String,
    /// One of image/jpeg, image/png or image/webp.
    pub mimeType: String,
    /// Nonempty typed binary bytes, at most 1048576; not a filesystem path or base64 string.
    pub content: Vec<u8>,
}

impl ImageUploadRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.filename);
        w.string(&self.mimeType);
        w.bytes(&self.content);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            filename: r.string()?,
            mimeType: r.string()?,
            content: r.bytes()?,
        })
    }
}

/// Playlist-scoped binary cover upload; no host filesystem path.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistImageUploadRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Basename only, 1..256 characters; no path separators, reserved filename punctuation or controls.
    pub filename: String,
    /// One of image/jpeg, image/png or image/webp.
    pub mimeType: String,
    /// Nonempty typed binary bytes, at most 1048576; the host snapshots the input before dispatch.
    pub content: Vec<u8>,
}

impl PlaylistImageUploadRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.filename);
        w.string(&self.mimeType);
        w.bytes(&self.content);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            filename: r.string()?,
            mimeType: r.string()?,
            content: r.bytes()?,
        })
    }
}

/// Uploaded image identity and optional public/display URL.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ImageUploadResult {
    /// Opaque uploaded-image identity, not a file path.
    pub imageId: String,
    /// Image URL.
    pub imageUrl: Option<String>,
}

impl ImageUploadResult {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.imageId);
        w.optional(self.imageUrl.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            imageId: r.string()?,
            imageUrl: r.optional(|r| r.string())?,
        })
    }
}

/// Structured import matching input; not a provider-local track identity.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ImportTrack {
    /// Display title; do not replace it with an ID.
    pub title: String,
    /// Artist name or attribution.
    pub artist: Option<String>,
    /// Album name.
    pub album: Option<String>,
}

impl ImportTrack {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.title);
        w.optional(self.artist.as_deref(), |w, s| w.string(s));
        w.optional(self.album.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            title: r.string()?,
            artist: r.optional(|r| r.string())?,
            album: r.optional(|r| r.string())?,
        })
    }
}

/// Import input with exactly one populated source: tracks, text or links.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistImportRequest {
    /// Display name.
    pub name: String,
    /// Structured matching input; exactly one of nonempty tracks, nonblank text or nonempty links is allowed.
    pub tracks: Vec<ImportTrack>,
    /// Nonblank source text when selected; null when another import source is used.
    pub text: Option<String>,
    /// Source links when selected; empty when another source is used.
    pub links: Vec<String>,
    /// Request import into the liked collection according to upstream support.
    pub importLiked: bool,
}

impl PlaylistImportRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.name);
        w.list(&self.tracks, |w, item| item.write(w));
        w.optional(self.text.as_deref(), |w, s| w.string(s));
        w.list(&self.links, |w, s| w.string(s));
        w.bool(self.importLiked);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            name: r.string()?,
            tracks: r.list(|r| ImportTrack::read(r))?,
            text: r.optional(|r| r.string())?,
            links: r.list(|r| r.string())?,
            importLiked: r.bool()?,
        })
    }
}

/// Pollable import task and known matching outcomes.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PlaylistImportTask {
    /// Opaque import-task identity; status polling must return the same identity.
    pub taskId: String,
    /// pending, running, completed or failed.
    pub status: String,
    /// Ordered playlist identities.
    pub playlistIds: Vec<String>,
    /// Known number of matched imports; null means unknown.
    pub matchedCount: Option<i32>,
    /// Known number of failed imports; null means unknown.
    pub failedCount: Option<i32>,
    /// Sanitized human-readable explanation, not a credential or upstream response dump.
    pub message: Option<String>,
}

impl PlaylistImportTask {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.taskId);
        w.string(&self.status);
        w.list(&self.playlistIds, |w, s| w.string(s));
        w.optional(self.matchedCount.as_ref(), |w, &v| w.int(v));
        w.optional(self.failedCount.as_ref(), |w, &v| w.int(v));
        w.optional(self.message.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            taskId: r.string()?,
            status: r.string()?,
            playlistIds: r.list(|r| r.string())?,
            matchedCount: r.optional(|r| r.int())?,
            failedCount: r.optional(|r| r.int())?,
            message: r.optional(|r| r.string())?,
        })
    }
}

/// Audio basename, positive file size, MD5 and bitrate for upload negotiation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudUploadPrepareRequest {
    /// Basename only, 1..256 characters; no path separators, reserved filename punctuation or controls.
    pub filename: String,
    /// Positive complete-file byte size, at most 9007199254740991.
    pub sizeBytes: i64,
    /// 32 hexadecimal characters for the complete file checksum, not a password hash.
    pub md5: String,
    /// Bitrate in bits per second, not kilobits per second.
    pub bitrate: i32,
}

impl CloudUploadPrepareRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.filename);
        w.long(self.sizeBytes);
        w.string(&self.md5);
        w.int(self.bitrate);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            filename: r.string()?,
            sizeBytes: r.long()?,
            md5: r.string()?,
            bitrate: r.int()?,
        })
    }
}

/// Opaque current-account-session upload handle and negotiated next position.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudUploadSession {
    /// Opaque account-session upload handle, never an upload token or host path.
    pub uploadId: String,
    /// Whether binary upload is needed after upstream deduplication/negotiation.
    pub needsUpload: bool,
    /// Next byte offset; must not exceed the declared complete-file size.
    pub offset: i64,
    /// Negotiated maximum chunk size, 1..1048576 bytes.
    pub chunkSize: i32,
}

impl CloudUploadSession {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.uploadId);
        w.bool(self.needsUpload);
        w.long(self.offset);
        w.int(self.chunkSize);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            uploadId: r.string()?,
            needsUpload: r.bool()?,
            offset: r.long()?,
            chunkSize: r.int()?,
        })
    }
}

/// One complete nonempty binary upload chunk at an explicit byte offset.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudUploadChunkRequest {
    /// Opaque account-session upload handle, never an upload token or host path.
    pub uploadId: String,
    /// Nonnegative byte offset; offset plus content length must be a safe integer.
    pub offset: i64,
    /// 1..1048576 typed binary bytes, no larger than the negotiated provider chunk size.
    pub content: Vec<u8>,
    /// True marks the final chunk; the provider verifies declared file-size boundaries.
    pub last: bool,
}

impl CloudUploadChunkRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.uploadId);
        w.long(self.offset);
        w.bytes(&self.content);
        w.bool(self.last);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            uploadId: r.string()?,
            offset: r.long()?,
            content: r.bytes()?,
            last: r.bool()?,
        })
    }
}

/// Exact whole-chunk acknowledgement for the same upload handle.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudUploadProgress {
    /// Opaque account-session upload handle, never an upload token or host path.
    pub uploadId: String,
    /// Exact next byte offset: request.offset + request.content byte length.
    pub offset: i64,
    /// Whole-chunk acknowledgement; must equal the input last flag.
    pub complete: bool,
}

impl CloudUploadProgress {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.uploadId);
        w.long(self.offset);
        w.bool(self.complete);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            uploadId: r.string()?,
            offset: r.long()?,
            complete: r.bool()?,
        })
    }
}

/// Upload publication input with optional track metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudUploadCompleteRequest {
    /// Opaque account-session upload handle, never an upload token or host path.
    pub uploadId: String,
    /// Display title; do not replace it with an ID.
    pub title: Option<String>,
    /// Artist name or attribution.
    pub artist: Option<String>,
    /// Album name.
    pub album: Option<String>,
}

impl CloudUploadCompleteRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.uploadId);
        w.optional(self.title.as_deref(), |w, s| w.string(s));
        w.optional(self.artist.as_deref(), |w, s| w.string(s));
        w.optional(self.album.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            uploadId: r.string()?,
            title: r.optional(|r| r.string())?,
            artist: r.optional(|r| r.string())?,
            album: r.optional(|r| r.string())?,
        })
    }
}

/// Explicit new/hot DJ-radio ranking selection and item-offset pagination.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DjRankingRequest {
    /// Ranking/chart selector: new or hot.
    pub kind: String,
    /// Requested item bound, 1..500; a response must not exceed it.
    pub limit: i32,
    /// Zero-based item offset, not a page number.
    pub offset: i32,
}

impl DjRankingRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.kind);
        w.int(self.limit);
        w.int(self.offset);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            kind: r.string()?,
            limit: r.int()?,
            offset: r.int()?,
        })
    }
}

/// Binding identity, provider-defined binding type and masked account information.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AccountBinding {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// Operation-specific discriminator; see the operation and record-specific constraints.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Display name.
    pub name: Option<String>,
    /// Masked public binding details only, never raw account credentials.
    pub maskedAccount: Option<String>,
}

impl AccountBinding {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.r#type);
        w.optional(self.name.as_deref(), |w, s| w.string(s));
        w.optional(self.maskedAccount.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            r#type: r.string()?,
            name: r.optional(|r| r.string())?,
            maskedAccount: r.optional(|r| r.string())?,
        })
    }
}

/// Current visible account bindings; never raw binding credentials.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AccountBindingPage {
    /// Ordered result items. Preserve upstream order; unknown pagination metadata stays null.
    pub items: Vec<AccountBinding>,
}

impl AccountBindingPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| AccountBinding::read(r))?,
        })
    }
}

/// New phone binding with country code, captcha and optional plaintext password.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PhoneBindingRequest {
    /// Digits-only phone input; extended binding requests require 4..20 digits.
    pub phone: String,
    /// Digits-only calling-country code, 1..4 digits, without a plus sign.
    pub countryCode: String,
    /// Captcha proof; do not log.
    pub captcha: String,
    /// Optional plaintext password; upstream hashing is the provider's responsibility.
    pub password: Option<String>,
}

impl PhoneBindingRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.phone);
        w.string(&self.countryCode);
        w.string(&self.captcha);
        w.optional(self.password.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            phone: r.string()?,
            countryCode: r.string()?,
            captcha: r.string()?,
            password: r.optional(|r| r.string())?,
        })
    }
}

/// Phone replacement with new-phone and old-phone captcha proofs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PhoneReplacementRequest {
    /// Digits-only phone input; extended binding requests require 4..20 digits.
    pub phone: String,
    /// Digits-only calling-country code, 1..4 digits, without a plus sign.
    pub countryCode: String,
    /// Captcha proof; do not log.
    pub captcha: String,
    /// Old-phone captcha proof, nonblank and at most 128 characters.
    pub oldCaptcha: String,
}

impl PhoneReplacementRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.phone);
        w.string(&self.countryCode);
        w.string(&self.captcha);
        w.string(&self.oldCaptcha);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            phone: r.string()?,
            countryCode: r.string()?,
            captcha: r.string()?,
            oldCaptcha: r.string()?,
        })
    }
}

/// Audio/MV/video identity and explicit media resource type.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MediaQualityRequest {
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// song, audio, mv or video; catalog containers such as playlist are invalid.
    #[serde(rename = "type")]
    pub r#type: String,
}

impl MediaQualityRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.r#type);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            r#type: r.string()?,
        })
    }
}

/// A unique provider-defined quality key, label and known availability/size/bitrate.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct QualityOption {
    /// Opaque provider quality key, not a CPMediaId or catalog resource identity.
    pub id: String,
    /// Human-readable quality label, separate from the provider-defined key.
    pub label: String,
    /// Bitrate in bits per second, not kilobits per second.
    pub bitrate: Option<i32>,
    /// Byte count; null means unknown where nullable.
    pub sizeBytes: Option<i64>,
    /// Whether this resource/quality is currently available to the account.
    pub available: bool,
}

impl QualityOption {
    pub fn write(&self, w: &mut WireWriter) {
        w.string(&self.id);
        w.string(&self.label);
        w.optional(self.bitrate.as_ref(), |w, &v| w.int(v));
        w.optional(self.sizeBytes.as_ref(), |w, &v| w.long(v));
        w.bool(self.available);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            id: r.string()?,
            label: r.string()?,
            bitrate: r.optional(|r| r.int())?,
            sizeBytes: r.optional(|r| r.long())?,
            available: r.bool()?,
        })
    }
}

/// Quality options with unique IDs; the application filters unavailable options.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct QualityPage {
    /// Quality options with unique IDs; unavailable options may be present and are filtered by the application.
    pub items: Vec<QualityOption>,
}

impl QualityPage {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.items, |w, item| item.write(w));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            items: r.list(|r| QualityOption::read(r))?,
        })
    }
}

/// Song, album or playlist sharing to private-message recipients.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResourceMessageRequest {
    /// Recipient provider-local user IDs, 1..100 where used by message requests.
    pub userIds: Vec<String>,
    /// Opaque, case-sensitive provider-local identity; preserve leading zeros and punctuation.
    pub id: String,
    /// song, album or playlist.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Plain text, never an upstream JSON message envelope.
    pub text: String,
}

impl ResourceMessageRequest {
    pub fn write(&self, w: &mut WireWriter) {
        w.list(&self.userIds, |w, s| w.string(s));
        w.string(&self.id);
        w.string(&self.r#type);
        w.string(&self.text);
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        Ok(Self {
            userIds: r.list(|r| r.string())?,
            id: r.string()?,
            r#type: r.string()?,
            text: r.string()?,
        })
    }
}

