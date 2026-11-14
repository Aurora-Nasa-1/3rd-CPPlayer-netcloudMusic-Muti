use ncm_api_rs::{create_client, Query};

#[tokio::main]
async fn main() {
    let client = create_client(None);
    let query = Query::new()
        .param("email", "notexist_probe@example.com")
        .param("password", "definitely_wrong_probe");
    match client.login(&query).await {
        Ok(resp) => println!("email login -> status={} body={}", resp.status, resp.body),
        Err(err) => println!("email login -> ERROR {err}"),
    }
}
