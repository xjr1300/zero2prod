use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    // 準備する。
    let address = spawn_app();
    // アプリケーションに対してHTTPリクエストを実行するための`requwest`を持ち出す必要がある。
    let client = reqwest::Client::new();

    // 実行
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");
    // 検証
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// バックグラウンドでアプリケーションを起動
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
