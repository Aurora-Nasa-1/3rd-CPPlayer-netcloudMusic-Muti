use ncm_api_rs::sdk::models::*;
use ncm_api_rs::sdk::operations::{all_capabilities, tags, PROTOCOL_VERSION};
use ncm_api_rs::sdk::provider::NeteaseSdkProvider;
use ncm_api_rs::sdk::wire::{WireReader, WireWriter};

#[tokio::test]
async fn test_sdk_capabilities_dispatch() {
    let provider = NeteaseSdkProvider::new("netease".into(), "default".into(), None, None);
    let in_bytes = {
        let mut w = WireWriter::new();
        Bytes { value: vec![] }.write(&mut w);
        w.finish()
    };
    let bytes = provider
        .dispatch(tags::CAPABILITIES, &in_bytes)
        .await
        .expect("dispatch capabilities failed");
    let mut reader = WireReader::new(&bytes).expect("reader new failed");
    let caps = Capabilities::read(&mut reader).expect("read capabilities failed");
    reader.finish().expect("finish failed");

    assert_eq!(PROTOCOL_VERSION, 1);
    assert_eq!(caps.supported.len(), all_capabilities().len());
    assert_eq!(caps.supported.len(), 162);
    assert!(caps.supported.contains(&"tracks.search".to_string()));
    assert!(caps.supported.contains(&"tracks.resolve".to_string()));
    assert!(caps.supported.contains(&"tracks.get".to_string()));
    assert!(caps.supported.contains(&"accounts.likes.page".to_string()));
    assert!(caps.supported.contains(&"social.likeResource".to_string()));
}

#[tokio::test]
async fn test_sdk_unsupported_operation() {
    let provider = NeteaseSdkProvider::new("netease".into(), "default".into(), None, None);
    let err = provider.dispatch(99999, &[]).await.unwrap_err();
    assert_eq!(err.category, "Unsupported");
}

#[tokio::test]
async fn test_sdk_models_roundtrip() {
    // 1. Failure
    let failure = Failure {
        category: "Network".to_string(),
        message: "Connection timeout".to_string(),
    };
    let mut w = WireWriter::new();
    failure.write(&mut w);
    let f_bytes = w.finish();
    let mut r = WireReader::new(&f_bytes).unwrap();
    assert_eq!(Failure::read(&mut r).unwrap(), failure);

    // 2. SearchRequest
    let req = SearchRequest {
        keyword: "周杰伦".to_string(),
        limit: 20,
        offset: 0,
    };
    let mut w = WireWriter::new();
    req.write(&mut w);
    let req_bytes = w.finish();
    let mut r = WireReader::new(&req_bytes).unwrap();
    assert_eq!(SearchRequest::read(&mut r).unwrap(), req);

    // 3. TrackPage & Track
    let track = Track {
        id: "123456".to_string(),
        title: "晴天".to_string(),
        artists: vec![Artist {
            id: Some("6452".to_string()),
            name: "周杰伦".to_string(),
        }],
        album: Some("叶惠美".to_string()),
        coverUrl: Some("https://example.com/cover.jpg".to_string()),
        durationMs: 269000,
    };
    let page = TrackPage {
        items: vec![track],
        total: Some(1),
        hasMore: Some(false),
    };
    let mut w = WireWriter::new();
    page.write(&mut w);
    let page_bytes = w.finish();
    let mut r = WireReader::new(&page_bytes).unwrap();
    assert_eq!(TrackPage::read(&mut r).unwrap(), page);

    // 4. IdPage
    let id_page = IdPage {
        items: vec!["1".to_string(), "2".to_string()],
        total: Some(2),
        hasMore: false,
    };
    let mut w = WireWriter::new();
    id_page.write(&mut w);
    let id_bytes = w.finish();
    let mut r = WireReader::new(&id_bytes).unwrap();
    assert_eq!(IdPage::read(&mut r).unwrap(), id_page);
}

#[test]
fn test_sdk_provider_cancellation() {
    let provider = NeteaseSdkProvider::new("netease".into(), "default".into(), None, None);
    assert!(!provider.is_cancelled());
    provider.cancel();
    assert!(provider.is_cancelled());
}

#[tokio::test]
async fn test_sdk_cancel_does_not_poison_later_ops() {
    let provider = NeteaseSdkProvider::new("netease".into(), "default".into(), None, None);
    provider.cancel();
    // 取消只针对「当时在途」的调用：下一次 dispatch 消费掉陈旧标记并正常执行，
    // 不允许出现「一次取消、之后全部 Operation cancelled」的毒化行为。
    let in_bytes = {
        let mut w = WireWriter::new();
        Bytes { value: vec![] }.write(&mut w);
        w.finish()
    };
    let bytes = provider
        .dispatch(tags::CAPABILITIES, &in_bytes)
        .await
        .expect("dispatch after cancel must succeed");
    assert!(!bytes.is_empty());
    assert!(
        !provider.is_cancelled(),
        "stale cancel flag must be consumed on dispatch"
    );
}

#[tokio::test]
async fn test_sdk_qr_flow() {
    let provider = NeteaseSdkProvider::new("netease".into(), "default".into(), None, None);

    // 1. qr.key
    let req_key = AuthActionRequest {
        action: "qr.key".to_string(),
        ..Default::default()
    };
    let mut w = WireWriter::new();
    req_key.write(&mut w);
    let bytes = provider
        .dispatch(tags::AUTHENTICATE, &w.finish())
        .await
        .expect("dispatch qr.key failed");
    let mut r = WireReader::new(&bytes).unwrap();
    let res_key = AuthActionResult::read(&mut r).unwrap();
    assert!(res_key.success, "qr.key should succeed");
    let key = res_key.key.expect("key must be present");
    assert!(!key.is_empty(), "key must not be empty");

    // 2. qr.create
    let req_create = AuthActionRequest {
        action: "qr.create".to_string(),
        key: Some(key.clone()),
        ..Default::default()
    };
    let mut w = WireWriter::new();
    req_create.write(&mut w);
    let bytes = provider
        .dispatch(tags::AUTHENTICATE, &w.finish())
        .await
        .expect("dispatch qr.create failed");
    let mut r = WireReader::new(&bytes).unwrap();
    let res_create = AuthActionResult::read(&mut r).unwrap();
    assert!(res_create.success, "qr.create should succeed");
    assert!(res_create.url.is_some(), "url must be present");

    // 3. qr.check
    let req_check = AuthActionRequest {
        action: "qr.check".to_string(),
        key: Some(key),
        ..Default::default()
    };
    let mut w = WireWriter::new();
    req_check.write(&mut w);
    let bytes = provider
        .dispatch(tags::AUTHENTICATE, &w.finish())
        .await
        .expect("dispatch qr.check failed");
    let mut r = WireReader::new(&bytes).unwrap();
    let res_check = AuthActionResult::read(&mut r).unwrap();
    assert_eq!(res_check.code, Some(801));
    assert_eq!(res_check.status, Some(801));
}

#[tokio::test]
async fn test_sdk_search_suggestions() {
    let provider = NeteaseSdkProvider::new("netease".into(), "default".into(), None, None);

    let req = SearchSuggestionRequest {
        keyword: "周杰伦".to_string(),
        limit: 5,
    };
    let mut w = WireWriter::new();
    req.write(&mut w);
    let bytes = provider
        .dispatch(tags::GETSEARCHSUGGESTIONS, &w.finish())
        .await
        .expect("dispatch search suggestions failed");
    let mut r = WireReader::new(&bytes).unwrap();
    let page = SuggestionPage::read(&mut r).unwrap();
    println!("search suggestions for 周杰伦: {:?}", page.items);
    assert!(
        !page.items.is_empty(),
        "search suggestions should not be empty"
    );
    assert!(page.items.len() <= 5, "should respect limit");
}

#[tokio::test]
async fn test_sdk_recommend_and_intelligence() {
    let client = ncm_api_rs::request::ApiClient::new(None);
    let mut q = ncm_api_rs::api::Query::new();
    q.params.insert("id".to_string(), "1361348080".to_string());
    q.params.insert("pid".to_string(), "24381616".to_string());
    q.params.insert("count".to_string(), "20".to_string());

    let mut opt_eapi = ncm_api_rs::request::RequestOption::default();
    opt_eapi.crypto = ncm_api_rs::request::CryptoType::Eapi;
    let res_eapi = client
        .request(
            "/api/playmode/intelligence/list",
            serde_json::json!({
                "songId": "1361348080",
                "type": "fromPlayOne",
                "playlistId": "24381616",
                "startMusicId": "1361348080",
                "count": 20
            }),
            opt_eapi,
        )
        .await;
    println!("intelligence Eapi result: {:?}", res_eapi);

    let mut opt_weapi = ncm_api_rs::request::RequestOption::default();
    opt_weapi.crypto = ncm_api_rs::request::CryptoType::Weapi;
    let res_weapi = client
        .request(
            "/api/playmode/intelligence/list",
            serde_json::json!({
                "songId": "1361348080",
                "type": "fromPlayOne",
                "playlistId": "24381616",
                "startMusicId": "1361348080",
                "count": 20
            }),
            opt_weapi,
        )
        .await;
    println!("intelligence Weapi result: {:?}", res_weapi);
}
