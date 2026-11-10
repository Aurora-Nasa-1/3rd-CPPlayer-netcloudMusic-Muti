import { writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import schema from "../../provider-contract/schema.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)));

const handlerMap = {
  // tracks
  search: "tracks::search_tracks",
  get: "tracks::get_track",
  getMany: "tracks::get_many_tracks",
  resolve: "tracks::resolve_track",
  getLyrics: "tracks::get_lyrics",
  checkTrackAvailable: "tracks::check_track_available",
  getMediaQualities: "tracks::get_media_qualities",
  scrobble: "tracks::scrobble_track",

  // playlists
  searchPlaylists: "playlists::search_playlists",
  getPlaylist: "playlists::get_playlist",
  getPlaylistTracks: "playlists::get_playlist_tracks",
  getUserPlaylists: "playlists::get_user_playlists",
  deletePlaylist: "playlists::delete_playlist",
  subscribePlaylist: "playlists::subscribe_playlist",
  addTracksToPlaylist: "playlists::add_tracks_to_playlist",
  removeTracksFromPlaylist: "playlists::remove_tracks_from_playlist",
  renamePlaylist: "playlists::rename_playlist",
  createPlaylist: "playlists::create_playlist",
  getSimilarPlaylists: "playlists::get_similar_playlists",
  updatePlaylist: "playlists::update_playlist",
  getPlaylistSubscribers: "playlists::get_playlist_subscribers",
  getPlaylistCategories: "playlists::get_playlist_categories",
  getHotPlaylistTags: "playlists::get_hot_playlist_tags",
  getHighqualityPlaylistTags: "playlists::get_highquality_playlist_tags",
  getCreatedPlaylists: "playlists::get_created_playlists",
  getCollectedPlaylists: "playlists::get_collected_playlists",
  reorderPlaylists: "playlists::reorder_playlists",
  reorderPlaylistTracks: "playlists::reorder_playlist_tracks",
  updatePlaylistCover: "playlists::update_playlist_cover",
  createPlaylistImport: "playlists::create_playlist_import",
  getPlaylistImportStatus: "playlists::get_playlist_import_status",

  // albums
  searchAlbums: "albums::search_albums",
  getAlbum: "albums::get_album",
  subscribeAlbum: "albums::subscribe_album",
  getDigitalAlbums: "albums::get_digital_albums",
  browseNewAlbums: "albums::browse_new_albums",
  getLatestAlbums: "albums::get_latest_albums",
  getAlbumChart: "albums::get_album_chart",
  getNewAlbums: "albums::get_new_albums",

  // artists
  searchArtists: "artists::search_artists",
  getArtist: "artists::get_artist",
  getArtistTracks: "artists::get_artist_tracks",
  getArtistAlbums: "artists::get_artist_albums",
  getHotArtists: "artists::get_hot_artists",
  getSimilarArtists: "artists::get_similar_artists",
  subscribeArtist: "artists::subscribe_artist",
  getSubscribedArtists: "artists::get_subscribed_artists",
  browseArtists: "artists::browse_artists",
  getArtistFollowCount: "artists::get_artist_follow_count",

  // discovery
  getRecommendedSongs: "discovery::get_recommended_songs",
  getRecommendedPlaylists: "discovery::get_recommended_playlists",
  getPersonalizedPlaylists: "discovery::get_personalized_playlists",
  getTopPlaylists: "discovery::get_top_playlists",
  getHighqualityPlaylists: "discovery::get_highquality_playlists",
  getPersonalizedNewSongs: "discovery::get_personalized_new_songs",
  getBanners: "discovery::get_banners",
  getRankings: "discovery::get_rankings",
  getTopSongs: "discovery::get_top_songs",
  getPersonalFm: "discovery::get_personal_fm",
  getSimilarSongs: "discovery::get_similar_songs",
  getIntelligenceSongs: "discovery::get_intelligence_songs",
  getHotSearches: "discovery::get_hot_searches",
  getSearchSuggestions: "discovery::get_search_suggestions",
  getRecommendationHistory: "discovery::get_recommendation_history",
  getRecommendationHistoryTracks: "discovery::get_recommendation_history_tracks",

  // accounts
  getUserProfile: "accounts::get_user_profile",
  getUserRecords: "accounts::get_user_records",
  getUserFollows: "accounts::get_user_follows",
  getUserFolloweds: "accounts::get_user_followeds",
  getUserCloud: "accounts::get_user_cloud",
  getLikeList: "accounts::get_like_list",
  getLikeListPage: "accounts::get_like_list_page",
  getUserAlbums: "accounts::get_user_albums",
  getCurrentAccount: "accounts::get_current_account",
  authenticate: "accounts::authenticate",
  updateUserProfile: "accounts::update_user_profile",
  dailySignin: "accounts::daily_signin",
  followUser: "accounts::follow_user",
  getRecentTracks: "accounts::get_recent_tracks",
  getRecentAlbums: "accounts::get_recent_albums",
  getRecentPlaylists: "accounts::get_recent_playlists",
  getRecentVideos: "accounts::get_recent_videos",
  getRecentDjRadios: "accounts::get_recent_dj_radios",
  cloudImport: "accounts::cloud_import",
  cloudMatch: "accounts::cloud_match",
  deleteUserCloud: "accounts::delete_user_cloud",
  getUserBindings: "accounts::get_user_bindings",
  bindPhone: "accounts::bind_phone",
  replacePhone: "accounts::replace_phone",
  likeSong: "accounts::like_song",
  dislikeSong: "accounts::dislike_song",

  // social & comments & messaging & listen together
  getComments: "social::get_comments",
  getFloorComments: "social::get_floor_comments",
  postComment: "social::post_comment",
  likeComment: "social::like_comment",
  deleteComment: "social::delete_comment",
  getCommentPage: "social::get_comment_page",
  getUserCommentHistory: "social::get_user_comment_history",
  getUnreadCount: "social::get_unread_count",
  getRecentContacts: "social::get_recent_contacts",
  getPrivateMessages: "social::get_private_messages",
  getMessageHistory: "social::get_message_history",
  markMessageAsRead: "social::mark_message_as_read",
  sendMessage: "social::send_message",
  sendResourceMessage: "social::send_resource_message",
  getCommentNotifications: "social::get_comment_notifications",
  getMentionNotifications: "social::get_mention_notifications",
  getSystemNotifications: "social::get_system_notifications",
  getCalendar: "social::get_calendar",
  getEvents: "social::get_events",
  getUserEvents: "social::get_user_events",
  deleteEvent: "social::delete_event",
  forwardEvent: "social::forward_event",
  shareResource: "social::share_resource",
  searchUsers: "social::search_users",
  getListenTogetherStatus: "social::get_listen_together_status",
  createListenTogetherRoom: "social::create_listen_together_room",
  checkListenTogetherRoom: "social::check_listen_together_room",
  acceptListenTogetherInvitation: "social::accept_listen_together_invitation",
  endListenTogether: "social::end_listen_together",
  heartbeatListenTogether: "social::heartbeat_listen_together",
  getListenTogetherSnapshot: "social::get_listen_together_snapshot",
  sendListenTogetherCommand: "social::send_listen_together_command",
  syncListenTogetherQueue: "social::sync_listen_together_queue",

  // media & videos & dj
  getMv: "media::get_mv",
  resolveMv: "media::resolve_mv",
  getAllMvs: "media::get_all_mvs",
  getLatestMvs: "media::get_latest_mvs",
  subscribeMv: "media::subscribe_mv",
  getSubscribedMvs: "media::get_subscribed_mvs",
  getVideo: "media::get_video",
  resolveVideo: "media::resolve_video",
  getVideoGroups: "media::get_video_groups",
  getVideoTimeline: "media::get_video_timeline",
  searchMvs: "media::search_mvs",
  searchVideos: "media::search_videos",
  getVideoCategories: "media::get_video_categories",
  getVideosByGroup: "media::get_videos_by_group",
  getRecommendedVideoTimeline: "media::get_recommended_video_timeline",
  subscribeVideo: "media::subscribe_video",
  getArtistMvs: "media::get_artist_mvs",
  getDjRadio: "media::get_dj_radio",
  getDjPrograms: "media::get_dj_programs",
  getHotDjRadios: "media::get_hot_dj_radios",
  getDjRanking: "media::get_dj_ranking",
  getRecommendedDjRadios: "media::get_recommended_dj_radios",
  subscribeDjRadio: "media::subscribe_dj_radio",
  getSubscribedDjRadios: "media::get_subscribed_dj_radios",
  getRecommendedPrograms: "media::get_recommended_programs",
  getDjProgram: "media::get_dj_program",
  getUserDjRadios: "media::get_user_dj_radios",
  searchDjRadios: "media::search_dj_radios",
  getDjCategories: "media::get_dj_categories",
  getDjRadiosByCategory: "media::get_dj_radios_by_category",
  getDjRadioSubscribers: "media::get_dj_radio_subscribers",
  browseDjRanking: "media::browse_dj_ranking",

  // uploads
  uploadAvatar: "uploads::upload_avatar",
  uploadPlaylistCover: "uploads::upload_playlist_cover",
  prepareCloudUpload: "uploads::prepare_cloud_upload",
  uploadCloudChunk: "uploads::upload_cloud_chunk",
  completeCloudUpload: "uploads::complete_cloud_upload"
};

// Check completeness
for (const [name, op] of Object.entries(schema.operations)) {
  if (['capabilities', 'http', 'sha256', 'storageGet', 'storageSet'].includes(name)) continue;
  if (!handlerMap[name]) {
    throw new Error(`Missing handler mapping for operation: ${name}`);
  }
}

let code = `// Generated from provider-contract/schema.mjs; do not edit.
#![allow(unused_imports, clippy::all)]

pub mod accounts;
pub mod albums;
pub mod artists;
pub mod discovery;
pub mod media;
pub mod parsers;
pub mod playlists;
pub mod social;
pub mod tracks;
pub mod uploads;

use crate::request::ApiClient;
use crate::sdk::models::*;
use crate::sdk::operations::{all_capabilities, tags};
use crate::sdk::wire::{WireReader, WireWriter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct NeteaseSdkProvider {
    pub client: ApiClient,
    pub provider_id: String,
    pub account_id: String,
    pub credential: Option<String>,
    pub cancel_flag: Arc<AtomicBool>,
}

impl NeteaseSdkProvider {
    pub fn new(
        provider_id: String,
        account_id: String,
        credential: Option<String>,
        _settings: Option<String>,
    ) -> Self {
        let client = ApiClient::new(credential.clone());
        Self {
            client,
            provider_id,
            account_id,
            credential,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    /// Execute one operation. \`cancel()\` interrupts only the invoke currently
    /// running (JNI invokes are serialized per session): the flag is consumed on
    /// exit, so one cancellation never poisons later operations.
    pub async fn dispatch(&self, operation: i32, in_bytes: &[u8]) -> Result<Vec<u8>, Failure> {
        // 入口消费掉遗留的取消标记（cancel() 只针对「当时在途」的调用）；
        // 执行期间的取消由下方 select! 打断，退出时再兜底清一次。
        self.cancel_flag.store(false, Ordering::SeqCst);
        let _reset = CancelReset(self.cancel_flag.clone());
        let flag = self.cancel_flag.clone();
        tokio::select! {
            biased;
            result = self.run_op(operation, in_bytes) => result,
            _ = async move {
                while !flag.load(Ordering::SeqCst) {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            } => Err(Failure {
                category: "Timeout".to_string(),
                message: "Operation cancelled".to_string(),
            }),
        }
    }

    async fn run_op(&self, operation: i32, in_bytes: &[u8]) -> Result<Vec<u8>, Failure> {
        let mut r = WireReader::new(in_bytes).map_err(|e| Failure {
            category: "Runtime".to_string(),
            message: format!("Wire decoding error: {}", e),
        })?;

        match operation {
            tags::CAPABILITIES => {
                let caps = Capabilities {
                    supported: all_capabilities().into_iter().map(String::from).collect(),
                };
                let mut w = WireWriter::new();
                caps.write(&mut w);
                Ok(w.finish())
            }
            tags::HTTP => Err(Failure {
                category: "Unsupported".to_string(),
                message: "Runtime HTTP is a host-side service; this provider performs its own upstream requests".to_string(),
            }),
            tags::SHA256 => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                use sha2::{Digest as _, Sha256};
                let hash = Sha256::digest(&req.value).to_vec();
                let resp = Bytes { value: hash };
                let mut w = WireWriter::new();
                resp.write(&mut w);
                Ok(w.finish())
            }
            tags::STORAGEGET | tags::STORAGESET => Err(Failure {
                category: "Unsupported".to_string(),
                message: "Runtime session storage is a host-side service for JS providers".to_string(),
            }),
`;

for (const [name, handler] of Object.entries(handlerMap)) {
  const op = schema.operations[name];
  code += `            tags::${name.toUpperCase()} => {\n`;
  code += `                let req = ${op.request}::read(&mut r).map_err(|e| Failure {\n`;
  code += `                    category: "Runtime".to_string(),\n`;
  code += `                    message: format!("Wire decode error: {}", e),\n`;
  code += `                })?;\n`;
  code += `                let res = ${handler}(&self.client, req).await?;\n`;
  code += `                let mut w = WireWriter::new();\n`;
  code += `                res.write(&mut w);\n`;
  code += `                Ok(w.finish())\n`;
  code += `            }\n`;
}

code += `            _ => Err(Failure {
                category: "Unsupported".to_string(),
                message: format!("Unsupported operation tag: {}", operation),
            }),
        }
    }
}

struct CancelReset(std::sync::Arc<AtomicBool>);

impl Drop for CancelReset {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
`;

writeFileSync(resolve(root, "src/sdk/provider/mod.rs"), code, "utf8");
console.log("src/sdk/provider/mod.rs written successfully.");
