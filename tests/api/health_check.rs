use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // 準備する。
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    // 実行
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");
    // 検証
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
