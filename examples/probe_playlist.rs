use ncm_api_rs::api::Query;
use ncm_api_rs::request::ApiClient;

#[tokio::main]
async fn main() {
    let client = ApiClient::new(None);
    for (limit, offset) in [
        (300, 0),
        (500, 0),
        (300, 300),
        (110, 300),
        (410, 0),
        (1000, 0),
    ] {
        let mut q = Query::new();
        q.params.insert("id".to_string(), "5315706876".to_string());
        q.params.insert("limit".to_string(), limit.to_string());
        q.params.insert("offset".to_string(), offset.to_string());
        match tokio::time::timeout(
            std::time::Duration::from_secs(20),
            client.playlist_track_all(&q),
        )
        .await
        {
            Ok(Ok(resp)) => {
                let songs = resp.body["songs"].as_array().map(|a| a.len()).unwrap_or(0);
                let ids = resp.body["playlist"]["trackIds"]
                    .as_array()
                    .map(|a| a.len())
                    .unwrap_or(0);
                println!(
                    "limit={limit} offset={offset} status={} songs={songs} trackIds={ids}",
                    resp.status
                );
            }
            Ok(Err(e)) => println!("limit={limit} offset={offset} ERR {e}"),
            Err(_) => println!("limit={limit} offset={offset} TIMEOUT"),
        }
    }
}
