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
    assert_eq!(caps.supported.len(), 161);
    assert!(caps.supported.contains(&"tracks.search".to_string()));
    assert!(caps.supported.contains(&"tracks.resolve".to_string()));
    assert!(caps.supported.contains(&"tracks.get".to_string()));
    assert!(caps.supported.contains(&"accounts.likes.page".to_string()));
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
