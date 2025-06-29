// In tests/links.rs
mod common;

use crate::common::spawn_app;
use axum_test::TestServer;
use backend::models::ShortLinkResponse;
use http::StatusCode;
use serde_json::json;
use url::Url;

#[tokio::test]
async fn create_link_and_redirect_works() {
    let test_app = spawn_app().await;
    let server = TestServer::new(test_app.app.clone()).unwrap();
    let original_url = "https://github.com/lukinhasram";

    let response = server
        .post("/shorten")
        .json(&json!({ "original_url": original_url }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: ShortLinkResponse = response.json();

    let short_url = Url::parse(&body.short_url).expect("short_url should be a valid URL");

    let redirect_response = server.get(short_url.path()).await;

    assert_eq!(redirect_response.status_code(), StatusCode::SEE_OTHER);
    let location_header = redirect_response
        .headers()
        .get("Location")
        .expect("Response should have a 'Location' header.")
        .to_str()
        .unwrap();
    assert_eq!(location_header, original_url);
}

#[tokio::test]
async fn create_link_returns_422_for_invalid_data() {
    let test_app = spawn_app().await;
    let server = TestServer::new(test_app.app.clone()).unwrap();
    let test_cases = vec![
        (json!({ "original_url": "" }), "empty url"),
        (json!({ "foo": "bar" }), "missing url field"),
        (
            json!({ "original_url": "not-a-valid-url" }),
            "invalid url format",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = server.post("/shorten").json(&invalid_body).await;

        assert_eq!(
            response.status_code(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not return a 422 when the payload was {}.",
            error_message
        );
    }
}
