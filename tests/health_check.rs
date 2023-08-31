#[tokio::test]
async fn health_check_works() {
    // 準備する。
    spawn_app();
    // アプリケーションに対してHTTPリクエストを実行するための`requwest`を持ち出す必要がある。
    let client = reqwest::Client::new();

    // 実行
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request");
    // 検証
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// バックグラウンドでアプリケーションを起動
fn spawn_app() {
    let server = zero2prod::run().expect("Failed to bind address");

    let _ = tokio::spawn(server);
}
