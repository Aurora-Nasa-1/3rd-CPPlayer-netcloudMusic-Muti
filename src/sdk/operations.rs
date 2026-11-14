// Generated from provider-contract/schema.mjs; do not edit.
#![allow(unused_imports, dead_code)]

pub const PROTOCOL_VERSION: i32 = 1;

pub mod tags {
    pub const CAPABILITIES: i32 = 1;
    pub const SEARCH: i32 = 2;
    pub const GET: i32 = 3;
    pub const GETMANY: i32 = 4;
    pub const RESOLVE: i32 = 5;
    pub const HTTP: i32 = 6;
    pub const SHA256: i32 = 7;
    pub const STORAGEGET: i32 = 8;
    pub const STORAGESET: i32 = 9;
    pub const GETLYRICS: i32 = 10;
    pub const SEARCHPLAYLISTS: i32 = 11;
    pub const GETPLAYLIST: i32 = 12;
    pub const GETPLAYLISTTRACKS: i32 = 13;
    pub const GETUSERPLAYLISTS: i32 = 14;
    pub const SEARCHALBUMS: i32 = 15;
    pub const GETALBUM: i32 = 16;
    pub const SEARCHARTISTS: i32 = 17;
    pub const GETARTIST: i32 = 18;
    pub const GETARTISTTRACKS: i32 = 19;
    pub const GETARTISTALBUMS: i32 = 20;
    pub const GETRECOMMENDEDSONGS: i32 = 21;
    pub const GETRECOMMENDEDPLAYLISTS: i32 = 22;
    pub const GETPERSONALIZEDPLAYLISTS: i32 = 23;
    pub const GETTOPPLAYLISTS: i32 = 24;
    pub const GETHIGHQUALITYPLAYLISTS: i32 = 25;
    pub const GETPERSONALIZEDNEWSONGS: i32 = 26;
    pub const GETBANNERS: i32 = 27;
    pub const GETRANKINGS: i32 = 28;
    pub const GETNEWALBUMS: i32 = 29;
    pub const GETTOPSONGS: i32 = 30;
    pub const GETHOTARTISTS: i32 = 31;
    pub const GETUSERPROFILE: i32 = 32;
    pub const GETUSERRECORDS: i32 = 33;
    pub const GETUSERFOLLOWS: i32 = 34;
    pub const GETUSERFOLLOWEDS: i32 = 35;
    pub const GETUSERCLOUD: i32 = 36;
    pub const GETLIKELIST: i32 = 37;
    pub const GETUSERALBUMS: i32 = 38;
    pub const DELETEPLAYLIST: i32 = 39;
    pub const SUBSCRIBEPLAYLIST: i32 = 40;
    pub const ADDTRACKSTOPLAYLIST: i32 = 41;
    pub const REMOVETRACKSFROMPLAYLIST: i32 = 42;
    pub const RENAMEPLAYLIST: i32 = 43;
    pub const CREATEPLAYLIST: i32 = 44;
    pub const LIKESONG: i32 = 45;
    pub const DISLIKESONG: i32 = 46;
    pub const DELETEUSERCLOUD: i32 = 47;
    pub const GETCOMMENTS: i32 = 48;
    pub const GETFLOORCOMMENTS: i32 = 49;
    pub const POSTCOMMENT: i32 = 50;
    pub const LIKECOMMENT: i32 = 51;
    pub const GETPERSONALFM: i32 = 52;
    pub const GETSIMILARSONGS: i32 = 53;
    pub const GETINTELLIGENCESONGS: i32 = 54;
    pub const SUBSCRIBEALBUM: i32 = 55;
    pub const GETHOTSEARCHES: i32 = 56;
    pub const GETSEARCHSUGGESTIONS: i32 = 57;
    pub const SCROBBLE: i32 = 58;
    pub const GETCURRENTACCOUNT: i32 = 59;
    pub const AUTHENTICATE: i32 = 60;
    pub const DELETECOMMENT: i32 = 61;
    pub const GETUNREADCOUNT: i32 = 62;
    pub const GETRECENTCONTACTS: i32 = 63;
    pub const GETPRIVATEMESSAGES: i32 = 64;
    pub const GETMESSAGEHISTORY: i32 = 65;
    pub const MARKMESSAGEASREAD: i32 = 66;
    pub const SENDMESSAGE: i32 = 67;
    pub const GETLISTENTOGETHERSTATUS: i32 = 68;
    pub const CREATELISTENTOGETHERROOM: i32 = 69;
    pub const CHECKLISTENTOGETHERROOM: i32 = 70;
    pub const ACCEPTLISTENTOGETHERINVITATION: i32 = 71;
    pub const ENDLISTENTOGETHER: i32 = 72;
    pub const HEARTBEATLISTENTOGETHER: i32 = 73;
    pub const GETLISTENTOGETHERSNAPSHOT: i32 = 74;
    pub const SENDLISTENTOGETHERCOMMAND: i32 = 75;
    pub const SYNCLISTENTOGETHERQUEUE: i32 = 76;
    pub const GETMV: i32 = 77;
    pub const RESOLVEMV: i32 = 78;
    pub const GETALLMVS: i32 = 79;
    pub const GETLATESTMVS: i32 = 80;
    pub const SUBSCRIBEMV: i32 = 81;
    pub const GETSUBSCRIBEDMVS: i32 = 82;
    pub const GETVIDEO: i32 = 83;
    pub const RESOLVEVIDEO: i32 = 84;
    pub const GETVIDEOGROUPS: i32 = 85;
    pub const GETVIDEOTIMELINE: i32 = 86;
    pub const GETDJRADIO: i32 = 87;
    pub const GETDJPROGRAMS: i32 = 88;
    pub const GETHOTDJRADIOS: i32 = 89;
    pub const GETDJRANKING: i32 = 90;
    pub const GETRECOMMENDEDDJRADIOS: i32 = 91;
    pub const SUBSCRIBEDJRADIO: i32 = 92;
    pub const GETSUBSCRIBEDDJRADIOS: i32 = 93;
    pub const GETRECOMMENDEDPROGRAMS: i32 = 94;
    pub const GETDJPROGRAM: i32 = 95;
    pub const GETUSERDJRADIOS: i32 = 96;
    pub const GETCALENDAR: i32 = 97;
    pub const GETEVENTS: i32 = 98;
    pub const GETUSEREVENTS: i32 = 99;
    pub const DELETEEVENT: i32 = 100;
    pub const FORWARDEVENT: i32 = 101;
    pub const SHARERESOURCE: i32 = 102;
    pub const UPDATEPLAYLIST: i32 = 103;
    pub const GETPLAYLISTSUBSCRIBERS: i32 = 104;
    pub const GETPLAYLISTCATEGORIES: i32 = 105;
    pub const GETHOTPLAYLISTTAGS: i32 = 106;
    pub const GETHIGHQUALITYPLAYLISTTAGS: i32 = 107;
    pub const GETCREATEDPLAYLISTS: i32 = 108;
    pub const GETCOLLECTEDPLAYLISTS: i32 = 109;
    pub const UPDATEUSERPROFILE: i32 = 110;
    pub const DAILYSIGNIN: i32 = 111;
    pub const FOLLOWUSER: i32 = 112;
    pub const GETRECENTTRACKS: i32 = 113;
    pub const CLOUDIMPORT: i32 = 114;
    pub const CLOUDMATCH: i32 = 115;
    pub const GETSIMILARARTISTS: i32 = 116;
    pub const GETSIMILARPLAYLISTS: i32 = 117;
    pub const SUBSCRIBEARTIST: i32 = 118;
    pub const GETSUBSCRIBEDARTISTS: i32 = 119;
    pub const GETARTISTMVS: i32 = 120;
    pub const BROWSEARTISTS: i32 = 121;
    pub const GETARTISTFOLLOWCOUNT: i32 = 122;
    pub const GETRECOMMENDATIONHISTORY: i32 = 123;
    pub const GETRECOMMENDATIONHISTORYTRACKS: i32 = 124;
    pub const GETDIGITALALBUMS: i32 = 125;
    pub const CHECKTRACKAVAILABLE: i32 = 126;
    pub const SEARCHMVS: i32 = 127;
    pub const SEARCHVIDEOS: i32 = 128;
    pub const SEARCHUSERS: i32 = 129;
    pub const SEARCHDJRADIOS: i32 = 130;
    pub const GETCOMMENTPAGE: i32 = 131;
    pub const BROWSENEWALBUMS: i32 = 132;
    pub const GETLATESTALBUMS: i32 = 133;
    pub const GETALBUMCHART: i32 = 134;
    pub const GETCOMMENTNOTIFICATIONS: i32 = 135;
    pub const GETMENTIONNOTIFICATIONS: i32 = 136;
    pub const GETSYSTEMNOTIFICATIONS: i32 = 137;
    pub const REORDERPLAYLISTS: i32 = 138;
    pub const REORDERPLAYLISTTRACKS: i32 = 139;
    pub const UPDATEPLAYLISTCOVER: i32 = 140;
    pub const UPLOADAVATAR: i32 = 141;
    pub const UPLOADPLAYLISTCOVER: i32 = 142;
    pub const CREATEPLAYLISTIMPORT: i32 = 143;
    pub const GETPLAYLISTIMPORTSTATUS: i32 = 144;
    pub const PREPARECLOUDUPLOAD: i32 = 145;
    pub const UPLOADCLOUDCHUNK: i32 = 146;
    pub const COMPLETECLOUDUPLOAD: i32 = 147;
    pub const GETVIDEOCATEGORIES: i32 = 148;
    pub const GETVIDEOSBYGROUP: i32 = 149;
    pub const GETRECOMMENDEDVIDEOTIMELINE: i32 = 150;
    pub const SUBSCRIBEVIDEO: i32 = 151;
    pub const GETDJCATEGORIES: i32 = 152;
    pub const GETDJRADIOSBYCATEGORY: i32 = 153;
    pub const GETDJRADIOSUBSCRIBERS: i32 = 154;
    pub const BROWSEDJRANKING: i32 = 155;
    pub const GETRECENTALBUMS: i32 = 156;
    pub const GETRECENTPLAYLISTS: i32 = 157;
    pub const GETRECENTVIDEOS: i32 = 158;
    pub const GETRECENTDJRADIOS: i32 = 159;
    pub const GETUSERBINDINGS: i32 = 160;
    pub const BINDPHONE: i32 = 161;
    pub const REPLACEPHONE: i32 = 162;
    pub const GETMEDIAQUALITIES: i32 = 163;
    pub const SENDRESOURCEMESSAGE: i32 = 164;
    pub const GETUSERCOMMENTHISTORY: i32 = 165;
    pub const GETLIKELISTPAGE: i32 = 166;
    pub const LIKERESOURCE: i32 = 167;
}

pub mod capabilities {
    pub const SEARCH: &str = "tracks.search";
    pub const GET: &str = "tracks.get";
    pub const GETMANY: &str = "tracks.getMany";
    pub const RESOLVE: &str = "tracks.resolve";
    pub const GETLYRICS: &str = "lyrics.get";
    pub const SEARCHPLAYLISTS: &str = "playlists.search";
    pub const GETPLAYLIST: &str = "playlists.get";
    pub const GETPLAYLISTTRACKS: &str = "playlists.tracks";
    pub const GETUSERPLAYLISTS: &str = "playlists.user";
    pub const SEARCHALBUMS: &str = "albums.search";
    pub const GETALBUM: &str = "albums.get";
    pub const SEARCHARTISTS: &str = "artists.search";
    pub const GETARTIST: &str = "artists.get";
    pub const GETARTISTTRACKS: &str = "artists.tracks";
    pub const GETARTISTALBUMS: &str = "artists.albums";
    pub const GETRECOMMENDEDSONGS: &str = "recommendations.songs";
    pub const GETRECOMMENDEDPLAYLISTS: &str = "recommendations.playlists";
    pub const GETPERSONALIZEDPLAYLISTS: &str = "recommendations.personalizedPlaylists";
    pub const GETTOPPLAYLISTS: &str = "recommendations.topPlaylists";
    pub const GETHIGHQUALITYPLAYLISTS: &str = "recommendations.highqualityPlaylists";
    pub const GETPERSONALIZEDNEWSONGS: &str = "recommendations.newSongs";
    pub const GETBANNERS: &str = "discovery.banners";
    pub const GETRANKINGS: &str = "discovery.rankings";
    pub const GETNEWALBUMS: &str = "discovery.newAlbums";
    pub const GETTOPSONGS: &str = "discovery.topSongs";
    pub const GETHOTARTISTS: &str = "discovery.hotArtists";
    pub const GETUSERPROFILE: &str = "accounts.profile";
    pub const GETUSERRECORDS: &str = "accounts.records";
    pub const GETUSERFOLLOWS: &str = "accounts.follows";
    pub const GETUSERFOLLOWEDS: &str = "accounts.followeds";
    pub const GETUSERCLOUD: &str = "accounts.cloud";
    pub const GETLIKELIST: &str = "accounts.likes";
    pub const GETUSERALBUMS: &str = "accounts.albums";
    pub const DELETEPLAYLIST: &str = "playlists.delete";
    pub const SUBSCRIBEPLAYLIST: &str = "playlists.subscribe";
    pub const ADDTRACKSTOPLAYLIST: &str = "playlists.addTracks";
    pub const REMOVETRACKSFROMPLAYLIST: &str = "playlists.removeTracks";
    pub const RENAMEPLAYLIST: &str = "playlists.rename";
    pub const CREATEPLAYLIST: &str = "playlists.create";
    pub const LIKESONG: &str = "library.like";
    pub const DISLIKESONG: &str = "library.dislike";
    pub const DELETEUSERCLOUD: &str = "library.cloud.delete";
    pub const GETCOMMENTS: &str = "comments.list";
    pub const GETFLOORCOMMENTS: &str = "comments.floor";
    pub const POSTCOMMENT: &str = "comments.post";
    pub const LIKECOMMENT: &str = "comments.like";
    pub const GETPERSONALFM: &str = "recommendations.personalFm";
    pub const GETSIMILARSONGS: &str = "recommendations.similarSongs";
    pub const GETINTELLIGENCESONGS: &str = "recommendations.intelligenceSongs";
    pub const SUBSCRIBEALBUM: &str = "albums.subscribe";
    pub const GETHOTSEARCHES: &str = "discovery.hotSearches";
    pub const GETSEARCHSUGGESTIONS: &str = "discovery.searchSuggestions";
    pub const SCROBBLE: &str = "playback.scrobble";
    pub const GETCURRENTACCOUNT: &str = "accounts.session";
    pub const AUTHENTICATE: &str = "auth.actions";
    pub const DELETECOMMENT: &str = "comments.delete";
    pub const GETUNREADCOUNT: &str = "messages.unread";
    pub const GETRECENTCONTACTS: &str = "messages.contacts";
    pub const GETPRIVATEMESSAGES: &str = "messages.conversations";
    pub const GETMESSAGEHISTORY: &str = "messages.history";
    pub const MARKMESSAGEASREAD: &str = "messages.read";
    pub const SENDMESSAGE: &str = "messages.send";
    pub const GETLISTENTOGETHERSTATUS: &str = "listenTogether.status";
    pub const CREATELISTENTOGETHERROOM: &str = "listenTogether.create";
    pub const CHECKLISTENTOGETHERROOM: &str = "listenTogether.check";
    pub const ACCEPTLISTENTOGETHERINVITATION: &str = "listenTogether.accept";
    pub const ENDLISTENTOGETHER: &str = "listenTogether.end";
    pub const HEARTBEATLISTENTOGETHER: &str = "listenTogether.heartbeat";
    pub const GETLISTENTOGETHERSNAPSHOT: &str = "listenTogether.snapshot";
    pub const SENDLISTENTOGETHERCOMMAND: &str = "listenTogether.command";
    pub const SYNCLISTENTOGETHERQUEUE: &str = "listenTogether.queue";
    pub const GETMV: &str = "mvs.get";
    pub const RESOLVEMV: &str = "mvs.resolve";
    pub const GETALLMVS: &str = "mvs.browse";
    pub const GETLATESTMVS: &str = "mvs.latest";
    pub const SUBSCRIBEMV: &str = "mvs.subscribe";
    pub const GETSUBSCRIBEDMVS: &str = "mvs.subscribed";
    pub const GETVIDEO: &str = "videos.get";
    pub const RESOLVEVIDEO: &str = "videos.resolve";
    pub const GETVIDEOGROUPS: &str = "videos.groups";
    pub const GETVIDEOTIMELINE: &str = "videos.timeline";
    pub const GETDJRADIO: &str = "dj.get";
    pub const GETDJPROGRAMS: &str = "dj.programs";
    pub const GETHOTDJRADIOS: &str = "dj.hot";
    pub const GETDJRANKING: &str = "dj.ranking";
    pub const GETRECOMMENDEDDJRADIOS: &str = "dj.recommended";
    pub const SUBSCRIBEDJRADIO: &str = "dj.subscribe";
    pub const GETSUBSCRIBEDDJRADIOS: &str = "dj.subscribed";
    pub const GETRECOMMENDEDPROGRAMS: &str = "dj.recommendedPrograms";
    pub const GETDJPROGRAM: &str = "dj.program";
    pub const GETUSERDJRADIOS: &str = "dj.user";
    pub const GETCALENDAR: &str = "social.calendar";
    pub const GETEVENTS: &str = "social.events";
    pub const GETUSEREVENTS: &str = "social.userEvents";
    pub const DELETEEVENT: &str = "social.deleteEvent";
    pub const FORWARDEVENT: &str = "social.forwardEvent";
    pub const SHARERESOURCE: &str = "social.share";
    pub const UPDATEPLAYLIST: &str = "playlists.update";
    pub const GETPLAYLISTSUBSCRIBERS: &str = "playlists.subscribers";
    pub const GETPLAYLISTCATEGORIES: &str = "playlists.categories";
    pub const GETHOTPLAYLISTTAGS: &str = "playlists.hotTags";
    pub const GETHIGHQUALITYPLAYLISTTAGS: &str = "playlists.highqualityTags";
    pub const GETCREATEDPLAYLISTS: &str = "playlists.created";
    pub const GETCOLLECTEDPLAYLISTS: &str = "playlists.collected";
    pub const UPDATEUSERPROFILE: &str = "accounts.update";
    pub const DAILYSIGNIN: &str = "accounts.signin";
    pub const FOLLOWUSER: &str = "accounts.follow";
    pub const GETRECENTTRACKS: &str = "accounts.recentTracks";
    pub const CLOUDIMPORT: &str = "accounts.cloud.import";
    pub const CLOUDMATCH: &str = "accounts.cloud.match";
    pub const GETSIMILARARTISTS: &str = "artists.similar";
    pub const GETSIMILARPLAYLISTS: &str = "playlists.similar";
    pub const SUBSCRIBEARTIST: &str = "artists.subscribe";
    pub const GETSUBSCRIBEDARTISTS: &str = "artists.subscribed";
    pub const GETARTISTMVS: &str = "artists.mvs";
    pub const BROWSEARTISTS: &str = "artists.browse";
    pub const GETARTISTFOLLOWCOUNT: &str = "artists.followCount";
    pub const GETRECOMMENDATIONHISTORY: &str = "recommendations.history";
    pub const GETRECOMMENDATIONHISTORYTRACKS: &str = "recommendations.historyTracks";
    pub const GETDIGITALALBUMS: &str = "albums.digital";
    pub const CHECKTRACKAVAILABLE: &str = "tracks.available";
    pub const SEARCHMVS: &str = "mvs.search";
    pub const SEARCHVIDEOS: &str = "videos.search";
    pub const SEARCHUSERS: &str = "users.search";
    pub const SEARCHDJRADIOS: &str = "dj.search";
    pub const GETCOMMENTPAGE: &str = "comments.page";
    pub const BROWSENEWALBUMS: &str = "albums.new";
    pub const GETLATESTALBUMS: &str = "albums.latest";
    pub const GETALBUMCHART: &str = "albums.chart";
    pub const GETCOMMENTNOTIFICATIONS: &str = "messages.comments";
    pub const GETMENTIONNOTIFICATIONS: &str = "messages.mentions";
    pub const GETSYSTEMNOTIFICATIONS: &str = "messages.notices";
    pub const REORDERPLAYLISTS: &str = "playlists.reorder";
    pub const REORDERPLAYLISTTRACKS: &str = "playlists.reorderTracks";
    pub const UPDATEPLAYLISTCOVER: &str = "playlists.cover";
    pub const UPLOADAVATAR: &str = "accounts.avatar.upload";
    pub const UPLOADPLAYLISTCOVER: &str = "playlists.cover.upload";
    pub const CREATEPLAYLISTIMPORT: &str = "playlists.import.create";
    pub const GETPLAYLISTIMPORTSTATUS: &str = "playlists.import.status";
    pub const PREPARECLOUDUPLOAD: &str = "library.cloud.upload.prepare";
    pub const UPLOADCLOUDCHUNK: &str = "library.cloud.upload.chunk";
    pub const COMPLETECLOUDUPLOAD: &str = "library.cloud.upload.complete";
    pub const GETVIDEOCATEGORIES: &str = "videos.categories";
    pub const GETVIDEOSBYGROUP: &str = "videos.group";
    pub const GETRECOMMENDEDVIDEOTIMELINE: &str = "videos.recommended";
    pub const SUBSCRIBEVIDEO: &str = "videos.subscribe";
    pub const GETDJCATEGORIES: &str = "dj.categories";
    pub const GETDJRADIOSBYCATEGORY: &str = "dj.category";
    pub const GETDJRADIOSUBSCRIBERS: &str = "dj.subscribers";
    pub const BROWSEDJRANKING: &str = "dj.ranking.browse";
    pub const GETRECENTALBUMS: &str = "accounts.recentAlbums";
    pub const GETRECENTPLAYLISTS: &str = "accounts.recentPlaylists";
    pub const GETRECENTVIDEOS: &str = "accounts.recentVideos";
    pub const GETRECENTDJRADIOS: &str = "accounts.recentDj";
    pub const GETUSERBINDINGS: &str = "accounts.bindings";
    pub const BINDPHONE: &str = "accounts.phone.bind";
    pub const REPLACEPHONE: &str = "accounts.phone.replace";
    pub const GETMEDIAQUALITIES: &str = "media.qualities";
    pub const SENDRESOURCEMESSAGE: &str = "messages.sendResource";
    pub const GETUSERCOMMENTHISTORY: &str = "comments.userHistory";
    pub const GETLIKELISTPAGE: &str = "accounts.likes.page";
    pub const LIKERESOURCE: &str = "social.likeResource";
}

/// Returns all capabilities supported by this SDK version.
pub fn all_capabilities() -> Vec<&'static str> {
    vec![
        capabilities::SEARCH,
        capabilities::GET,
        capabilities::GETMANY,
        capabilities::RESOLVE,
        capabilities::GETLYRICS,
        capabilities::SEARCHPLAYLISTS,
        capabilities::GETPLAYLIST,
        capabilities::GETPLAYLISTTRACKS,
        capabilities::GETUSERPLAYLISTS,
        capabilities::SEARCHALBUMS,
        capabilities::GETALBUM,
        capabilities::SEARCHARTISTS,
        capabilities::GETARTIST,
        capabilities::GETARTISTTRACKS,
        capabilities::GETARTISTALBUMS,
        capabilities::GETRECOMMENDEDSONGS,
        capabilities::GETRECOMMENDEDPLAYLISTS,
        capabilities::GETPERSONALIZEDPLAYLISTS,
        capabilities::GETTOPPLAYLISTS,
        capabilities::GETHIGHQUALITYPLAYLISTS,
        capabilities::GETPERSONALIZEDNEWSONGS,
        capabilities::GETBANNERS,
        capabilities::GETRANKINGS,
        capabilities::GETNEWALBUMS,
        capabilities::GETTOPSONGS,
        capabilities::GETHOTARTISTS,
        capabilities::GETUSERPROFILE,
        capabilities::GETUSERRECORDS,
        capabilities::GETUSERFOLLOWS,
        capabilities::GETUSERFOLLOWEDS,
        capabilities::GETUSERCLOUD,
        capabilities::GETLIKELIST,
        capabilities::GETUSERALBUMS,
        capabilities::DELETEPLAYLIST,
        capabilities::SUBSCRIBEPLAYLIST,
        capabilities::ADDTRACKSTOPLAYLIST,
        capabilities::REMOVETRACKSFROMPLAYLIST,
        capabilities::RENAMEPLAYLIST,
        capabilities::CREATEPLAYLIST,
        capabilities::LIKESONG,
        capabilities::DISLIKESONG,
        capabilities::DELETEUSERCLOUD,
        capabilities::GETCOMMENTS,
        capabilities::GETFLOORCOMMENTS,
        capabilities::POSTCOMMENT,
        capabilities::LIKECOMMENT,
        capabilities::GETPERSONALFM,
        capabilities::GETSIMILARSONGS,
        capabilities::GETINTELLIGENCESONGS,
        capabilities::SUBSCRIBEALBUM,
        capabilities::GETHOTSEARCHES,
        capabilities::GETSEARCHSUGGESTIONS,
        capabilities::SCROBBLE,
        capabilities::GETCURRENTACCOUNT,
        capabilities::AUTHENTICATE,
        capabilities::DELETECOMMENT,
        capabilities::GETUNREADCOUNT,
        capabilities::GETRECENTCONTACTS,
        capabilities::GETPRIVATEMESSAGES,
        capabilities::GETMESSAGEHISTORY,
        capabilities::MARKMESSAGEASREAD,
        capabilities::SENDMESSAGE,
        capabilities::GETLISTENTOGETHERSTATUS,
        capabilities::CREATELISTENTOGETHERROOM,
        capabilities::CHECKLISTENTOGETHERROOM,
        capabilities::ACCEPTLISTENTOGETHERINVITATION,
        capabilities::ENDLISTENTOGETHER,
        capabilities::HEARTBEATLISTENTOGETHER,
        capabilities::GETLISTENTOGETHERSNAPSHOT,
        capabilities::SENDLISTENTOGETHERCOMMAND,
        capabilities::SYNCLISTENTOGETHERQUEUE,
        capabilities::GETMV,
        capabilities::RESOLVEMV,
        capabilities::GETALLMVS,
        capabilities::GETLATESTMVS,
        capabilities::SUBSCRIBEMV,
        capabilities::GETSUBSCRIBEDMVS,
        capabilities::GETVIDEO,
        capabilities::RESOLVEVIDEO,
        capabilities::GETVIDEOGROUPS,
        capabilities::GETVIDEOTIMELINE,
        capabilities::GETDJRADIO,
        capabilities::GETDJPROGRAMS,
        capabilities::GETHOTDJRADIOS,
        capabilities::GETDJRANKING,
        capabilities::GETRECOMMENDEDDJRADIOS,
        capabilities::SUBSCRIBEDJRADIO,
        capabilities::GETSUBSCRIBEDDJRADIOS,
        capabilities::GETRECOMMENDEDPROGRAMS,
        capabilities::GETDJPROGRAM,
        capabilities::GETUSERDJRADIOS,
        capabilities::GETCALENDAR,
        capabilities::GETEVENTS,
        capabilities::GETUSEREVENTS,
        capabilities::DELETEEVENT,
        capabilities::FORWARDEVENT,
        capabilities::SHARERESOURCE,
        capabilities::UPDATEPLAYLIST,
        capabilities::GETPLAYLISTSUBSCRIBERS,
        capabilities::GETPLAYLISTCATEGORIES,
        capabilities::GETHOTPLAYLISTTAGS,
        capabilities::GETHIGHQUALITYPLAYLISTTAGS,
        capabilities::GETCREATEDPLAYLISTS,
        capabilities::GETCOLLECTEDPLAYLISTS,
        capabilities::UPDATEUSERPROFILE,
        capabilities::DAILYSIGNIN,
        capabilities::FOLLOWUSER,
        capabilities::GETRECENTTRACKS,
        capabilities::CLOUDIMPORT,
        capabilities::CLOUDMATCH,
        capabilities::GETSIMILARARTISTS,
        capabilities::GETSIMILARPLAYLISTS,
        capabilities::SUBSCRIBEARTIST,
        capabilities::GETSUBSCRIBEDARTISTS,
        capabilities::GETARTISTMVS,
        capabilities::BROWSEARTISTS,
        capabilities::GETARTISTFOLLOWCOUNT,
        capabilities::GETRECOMMENDATIONHISTORY,
        capabilities::GETRECOMMENDATIONHISTORYTRACKS,
        capabilities::GETDIGITALALBUMS,
        capabilities::CHECKTRACKAVAILABLE,
        capabilities::SEARCHMVS,
        capabilities::SEARCHVIDEOS,
        capabilities::SEARCHUSERS,
        capabilities::SEARCHDJRADIOS,
        capabilities::GETCOMMENTPAGE,
        capabilities::BROWSENEWALBUMS,
        capabilities::GETLATESTALBUMS,
        capabilities::GETALBUMCHART,
        capabilities::GETCOMMENTNOTIFICATIONS,
        capabilities::GETMENTIONNOTIFICATIONS,
        capabilities::GETSYSTEMNOTIFICATIONS,
        capabilities::REORDERPLAYLISTS,
        capabilities::REORDERPLAYLISTTRACKS,
        capabilities::UPDATEPLAYLISTCOVER,
        capabilities::UPLOADAVATAR,
        capabilities::UPLOADPLAYLISTCOVER,
        capabilities::CREATEPLAYLISTIMPORT,
        capabilities::GETPLAYLISTIMPORTSTATUS,
        capabilities::PREPARECLOUDUPLOAD,
        capabilities::UPLOADCLOUDCHUNK,
        capabilities::COMPLETECLOUDUPLOAD,
        capabilities::GETVIDEOCATEGORIES,
        capabilities::GETVIDEOSBYGROUP,
        capabilities::GETRECOMMENDEDVIDEOTIMELINE,
        capabilities::SUBSCRIBEVIDEO,
        capabilities::GETDJCATEGORIES,
        capabilities::GETDJRADIOSBYCATEGORY,
        capabilities::GETDJRADIOSUBSCRIBERS,
        capabilities::BROWSEDJRANKING,
        capabilities::GETRECENTALBUMS,
        capabilities::GETRECENTPLAYLISTS,
        capabilities::GETRECENTVIDEOS,
        capabilities::GETRECENTDJRADIOS,
        capabilities::GETUSERBINDINGS,
        capabilities::BINDPHONE,
        capabilities::REPLACEPHONE,
        capabilities::GETMEDIAQUALITIES,
        capabilities::SENDRESOURCEMESSAGE,
        capabilities::GETUSERCOMMENTHISTORY,
        capabilities::GETLIKELISTPAGE,
        capabilities::LIKERESOURCE,
    ]
}
