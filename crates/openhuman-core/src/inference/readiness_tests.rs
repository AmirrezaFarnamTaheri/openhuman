use super::*;
use crate::security::credentials::{AuthService, APP_SESSION_PROVIDER, DEFAULT_AUTH_PROFILE_NAME};
use tempfile::TempDir;

#[tokio::test]
async fn cached_probe_serves_definitive_negative_results() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let tmp = TempDir::new().unwrap();
    let mut config = Config {
        workspace_dir: tmp.path().join("workspace"),
        action_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config.toml"),
        ..Config::default()
    };
    AuthService::new(tmp.path(), false)
        .store_provider_token(
            APP_SESSION_PROVIDER,
            DEFAULT_AUTH_PROFILE_NAME,
            "test.session.jwt",
            std::collections::HashMap::new(),
            true,
        )
        .expect("seed app-session token");
    let hit_count = std::sync::Arc::new(AtomicUsize::new(0));
    let counter = hit_count.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = axum::Router::new().route(
        "/openai/v1/chat/completions",
        axum::routing::post(move || {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                use axum::response::IntoResponse;
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({
                        "success": false,
                        "error": "API key not configured for provider",
                        "errorCode": "BAD_REQUEST"
                    })),
                )
                    .into_response()
            }
        }),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    config.api_url = Some(format!("http://{addr}"));

    let first = cached_probe(&DispatchCtx::ambient(), "summarization", &config).await;
    assert!(first
        .unwrap_err()
        .to_ascii_lowercase()
        .contains("api key not configured for provider"));
    assert_eq!(hit_count.load(Ordering::SeqCst), 1);
    assert!(
        cached_probe(&DispatchCtx::ambient(), "summarization", &config)
            .await
            .is_err()
    );
    assert_eq!(hit_count.load(Ordering::SeqCst), 1);
}
