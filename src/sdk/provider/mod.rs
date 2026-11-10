// Generated from provider-contract/schema.mjs; do not edit.
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

    pub async fn dispatch(&self, operation: i32, in_bytes: &[u8]) -> Result<Vec<u8>, Failure> {
        if self.cancel_flag.load(Ordering::SeqCst) {
            return Err(Failure {
                category: "Timeout".to_string(),
                message: "Operation cancelled".to_string(),
            });
        }

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
            tags::HTTP => {
                let _req = HttpRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let resp = HttpResponse {
                    status: 200,
                    headers: vec![],
                    body: vec![],
                };
                let mut w = WireWriter::new();
                resp.write(&mut w);
                Ok(w.finish())
            }
            tags::SHA256 => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                use md5::Digest;
                let hash = md5::Md5::digest(&req.value).to_vec();
                let resp = Bytes { value: hash };
                let mut w = WireWriter::new();
                resp.write(&mut w);
                Ok(w.finish())
            }
            tags::STORAGEGET => {
                let _req = StorageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let resp = StorageValue { value: None };
                let mut w = WireWriter::new();
                resp.write(&mut w);
                Ok(w.finish())
            }
            tags::STORAGESET => {
                let _req = StorageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let resp = StorageValue { value: None };
                let mut w = WireWriter::new();
                resp.write(&mut w);
                Ok(w.finish())
            }
            tags::SEARCH => {
                let req = SearchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = tracks::search_tracks(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GET => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = tracks::get_track(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETMANY => {
                let req = BatchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = tracks::get_many_tracks(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::RESOLVE => {
                let req = ResolveRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = tracks::resolve_track(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETLYRICS => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = tracks::get_lyrics(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::CHECKTRACKAVAILABLE => {
                let req = TrackAvailabilityRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = tracks::check_track_available(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETMEDIAQUALITIES => {
                let req = MediaQualityRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = tracks::get_media_qualities(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SCROBBLE => {
                let req = ScrobbleRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = tracks::scrobble_track(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SEARCHPLAYLISTS => {
                let req = SearchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::search_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPLAYLIST => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_playlist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPLAYLISTTRACKS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_playlist_tracks(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERPLAYLISTS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_user_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::DELETEPLAYLIST => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::delete_playlist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SUBSCRIBEPLAYLIST => {
                let req = PlaylistSubscribeRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::subscribe_playlist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::ADDTRACKSTOPLAYLIST => {
                let req = PlaylistTracksRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::add_tracks_to_playlist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::REMOVETRACKSFROMPLAYLIST => {
                let req = PlaylistTracksRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::remove_tracks_from_playlist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::RENAMEPLAYLIST => {
                let req = PlaylistNameRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::rename_playlist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::CREATEPLAYLIST => {
                let req = CreatePlaylistRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::create_playlist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETSIMILARPLAYLISTS => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_similar_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::UPDATEPLAYLIST => {
                let req = PlaylistUpdateRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::update_playlist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPLAYLISTSUBSCRIBERS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_playlist_subscribers(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPLAYLISTCATEGORIES => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_playlist_categories(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETHOTPLAYLISTTAGS => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_hot_playlist_tags(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETHIGHQUALITYPLAYLISTTAGS => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_highquality_playlist_tags(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETCREATEDPLAYLISTS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_created_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETCOLLECTEDPLAYLISTS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_collected_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::REORDERPLAYLISTS => {
                let req = PlaylistOrderRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::reorder_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::REORDERPLAYLISTTRACKS => {
                let req = PlaylistTrackOrderRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::reorder_playlist_tracks(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::UPDATEPLAYLISTCOVER => {
                let req = PlaylistCoverRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::update_playlist_cover(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::CREATEPLAYLISTIMPORT => {
                let req = PlaylistImportRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::create_playlist_import(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPLAYLISTIMPORTSTATUS => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = playlists::get_playlist_import_status(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SEARCHALBUMS => {
                let req = SearchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = albums::search_albums(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETALBUM => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = albums::get_album(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SUBSCRIBEALBUM => {
                let req = AlbumSubscribeRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = albums::subscribe_album(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETDIGITALALBUMS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = albums::get_digital_albums(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::BROWSENEWALBUMS => {
                let req = AlbumPageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = albums::browse_new_albums(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETLATESTALBUMS => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = albums::get_latest_albums(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETALBUMCHART => {
                let req = AlbumChartRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = albums::get_album_chart(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETNEWALBUMS => {
                let req = AlbumBrowseRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = albums::get_new_albums(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SEARCHARTISTS => {
                let req = SearchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::search_artists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETARTIST => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::get_artist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETARTISTTRACKS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::get_artist_tracks(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETARTISTALBUMS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::get_artist_albums(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETHOTARTISTS => {
                let req = RecommendationsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::get_hot_artists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETSIMILARARTISTS => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::get_similar_artists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SUBSCRIBEARTIST => {
                let req = SubscriptionRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::subscribe_artist(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETSUBSCRIBEDARTISTS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::get_subscribed_artists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::BROWSEARTISTS => {
                let req = ArtistBrowseRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::browse_artists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETARTISTFOLLOWCOUNT => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = artists::get_artist_follow_count(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECOMMENDEDSONGS => {
                let req = RecommendationsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_recommended_songs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECOMMENDEDPLAYLISTS => {
                let req = RecommendationsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_recommended_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPERSONALIZEDPLAYLISTS => {
                let req = RecommendationsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_personalized_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETTOPPLAYLISTS => {
                let req = PlaylistBrowseRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_top_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETHIGHQUALITYPLAYLISTS => {
                let req = PlaylistBrowseRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_highquality_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPERSONALIZEDNEWSONGS => {
                let req = RecommendationsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_personalized_new_songs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETBANNERS => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_banners(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRANKINGS => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_rankings(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETTOPSONGS => {
                let req = TopSongsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_top_songs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPERSONALFM => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_personal_fm(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETSIMILARSONGS => {
                let req = SimilarSongsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_similar_songs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETINTELLIGENCESONGS => {
                let req = IntelligenceSongsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_intelligence_songs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETHOTSEARCHES => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_hot_searches(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETSEARCHSUGGESTIONS => {
                let req = SearchSuggestionRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_search_suggestions(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECOMMENDATIONHISTORY => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_recommendation_history(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECOMMENDATIONHISTORYTRACKS => {
                let req = HistoryTracksRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = discovery::get_recommendation_history_tracks(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERPROFILE => {
                let req = UserRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_user_profile(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERRECORDS => {
                let req = UserRecordsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_user_records(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERFOLLOWS => {
                let req = UserListRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_user_follows(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERFOLLOWEDS => {
                let req = UserListRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_user_followeds(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERCLOUD => {
                let req = UserListRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_user_cloud(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETLIKELIST => {
                let req = UserRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_like_list(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETLIKELISTPAGE => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_like_list_page(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERALBUMS => {
                let req = UserListRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_user_albums(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETCURRENTACCOUNT => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_current_account(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::AUTHENTICATE => {
                let req = AuthActionRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::authenticate(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::UPDATEUSERPROFILE => {
                let req = UserProfileUpdateRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::update_user_profile(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::DAILYSIGNIN => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::daily_signin(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::FOLLOWUSER => {
                let req = FollowUserRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::follow_user(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECENTTRACKS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_recent_tracks(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECENTALBUMS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_recent_albums(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECENTPLAYLISTS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_recent_playlists(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECENTVIDEOS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_recent_videos(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECENTDJRADIOS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_recent_dj_radios(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::CLOUDIMPORT => {
                let req = CloudImportRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::cloud_import(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::CLOUDMATCH => {
                let req = CloudMatchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::cloud_match(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::DELETEUSERCLOUD => {
                let req = BatchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::delete_user_cloud(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERBINDINGS => {
                let req = UserRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::get_user_bindings(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::BINDPHONE => {
                let req = PhoneBindingRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::bind_phone(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::REPLACEPHONE => {
                let req = PhoneReplacementRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::replace_phone(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::LIKESONG => {
                let req = SongLikeRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::like_song(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::DISLIKESONG => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = accounts::dislike_song(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETCOMMENTS => {
                let req = CommentPageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_comments(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETFLOORCOMMENTS => {
                let req = CommentFloorRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_floor_comments(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::POSTCOMMENT => {
                let req = PostCommentRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::post_comment(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::LIKECOMMENT => {
                let req = CommentLikeRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::like_comment(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::DELETECOMMENT => {
                let req = DeleteCommentRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::delete_comment(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETCOMMENTPAGE => {
                let req = CommentQuery::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_comment_page(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERCOMMENTHISTORY => {
                let req = UserCursorRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_user_comment_history(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUNREADCOUNT => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_unread_count(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECENTCONTACTS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_recent_contacts(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETPRIVATEMESSAGES => {
                let req = CursorRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_private_messages(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETMESSAGEHISTORY => {
                let req = MessageHistoryRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_message_history(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::MARKMESSAGEASREAD => {
                let req = UserRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::mark_message_as_read(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SENDMESSAGE => {
                let req = SendMessageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::send_message(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SENDRESOURCEMESSAGE => {
                let req = ResourceMessageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::send_resource_message(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETCOMMENTNOTIFICATIONS => {
                let req = UserCursorRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_comment_notifications(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETMENTIONNOTIFICATIONS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_mention_notifications(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETSYSTEMNOTIFICATIONS => {
                let req = CursorRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_system_notifications(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETCALENDAR => {
                let req = CalendarRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_calendar(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETEVENTS => {
                let req = EventPageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_events(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSEREVENTS => {
                let req = UserEventRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_user_events(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::DELETEEVENT => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::delete_event(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::FORWARDEVENT => {
                let req = ForwardEventRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::forward_event(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SHARERESOURCE => {
                let req = ShareResourceRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::share_resource(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SEARCHUSERS => {
                let req = SearchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::search_users(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETLISTENTOGETHERSTATUS => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_listen_together_status(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::CREATELISTENTOGETHERROOM => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::create_listen_together_room(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::CHECKLISTENTOGETHERROOM => {
                let req = ListenInvitationRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::check_listen_together_room(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::ACCEPTLISTENTOGETHERINVITATION => {
                let req = ListenInvitationRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::accept_listen_together_invitation(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::ENDLISTENTOGETHER => {
                let req = ListenRoomRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::end_listen_together(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::HEARTBEATLISTENTOGETHER => {
                let req = ListenHeartbeatRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::heartbeat_listen_together(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETLISTENTOGETHERSNAPSHOT => {
                let req = ListenRoomRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::get_listen_together_snapshot(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SENDLISTENTOGETHERCOMMAND => {
                let req = ListenCommandRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::send_listen_together_command(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SYNCLISTENTOGETHERQUEUE => {
                let req = ListenQueueRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = social::sync_listen_together_queue(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETMV => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_mv(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::RESOLVEMV => {
                let req = VideoResolveRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::resolve_mv(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETALLMVS => {
                let req = VideoBrowseRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_all_mvs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETLATESTMVS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_latest_mvs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SUBSCRIBEMV => {
                let req = SubscriptionRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::subscribe_mv(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETSUBSCRIBEDMVS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_subscribed_mvs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETVIDEO => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_video(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::RESOLVEVIDEO => {
                let req = VideoResolveRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::resolve_video(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETVIDEOGROUPS => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_video_groups(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETVIDEOTIMELINE => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_video_timeline(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SEARCHMVS => {
                let req = SearchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::search_mvs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SEARCHVIDEOS => {
                let req = SearchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::search_videos(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETVIDEOCATEGORIES => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_video_categories(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETVIDEOSBYGROUP => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_videos_by_group(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECOMMENDEDVIDEOTIMELINE => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_recommended_video_timeline(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SUBSCRIBEVIDEO => {
                let req = SubscriptionRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::subscribe_video(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETARTISTMVS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_artist_mvs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETDJRADIO => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_dj_radio(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETDJPROGRAMS => {
                let req = DjProgramsRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_dj_programs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETHOTDJRADIOS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_hot_dj_radios(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETDJRANKING => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_dj_ranking(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECOMMENDEDDJRADIOS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_recommended_dj_radios(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SUBSCRIBEDJRADIO => {
                let req = SubscriptionRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::subscribe_dj_radio(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETSUBSCRIBEDDJRADIOS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_subscribed_dj_radios(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETRECOMMENDEDPROGRAMS => {
                let req = PageRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_recommended_programs(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETDJPROGRAM => {
                let req = IdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_dj_program(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETUSERDJRADIOS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_user_dj_radios(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::SEARCHDJRADIOS => {
                let req = SearchRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::search_dj_radios(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETDJCATEGORIES => {
                let req = Bytes::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_dj_categories(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETDJRADIOSBYCATEGORY => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_dj_radios_by_category(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::GETDJRADIOSUBSCRIBERS => {
                let req = PagedIdRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::get_dj_radio_subscribers(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::BROWSEDJRANKING => {
                let req = DjRankingRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = media::browse_dj_ranking(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::UPLOADAVATAR => {
                let req = ImageUploadRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = uploads::upload_avatar(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::UPLOADPLAYLISTCOVER => {
                let req = PlaylistImageUploadRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = uploads::upload_playlist_cover(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::PREPARECLOUDUPLOAD => {
                let req = CloudUploadPrepareRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = uploads::prepare_cloud_upload(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::UPLOADCLOUDCHUNK => {
                let req = CloudUploadChunkRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = uploads::upload_cloud_chunk(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            tags::COMPLETECLOUDUPLOAD => {
                let req = CloudUploadCompleteRequest::read(&mut r).map_err(|e| Failure {
                    category: "Runtime".to_string(),
                    message: format!("Wire decode error: {}", e),
                })?;
                let res = uploads::complete_cloud_upload(&self.client, req).await?;
                let mut w = WireWriter::new();
                res.write(&mut w);
                Ok(w.finish())
            }
            _ => Err(Failure {
                category: "Unsupported".to_string(),
                message: format!("Unsupported operation tag: {}", operation),
            }),
        }
    }
}
