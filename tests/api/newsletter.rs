use std::time::Duration;

use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use uuid::Uuid;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};

/// テストの元でアプリケーションの公開APIを、確認していない購読者を作成するために使用
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    // ここで、複数の購読者で作業する。
    // 購読者の詳細は衝突を避けるためにランダムにする必要がある。
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email,
    }))
    .unwrap();

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        // `mount`メソッドではなく、`mount_as_scoped`メソッドを使用
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    // Postmarkサーバーのモックから受け取ったリクエスをと検証して、
    // 確認リンクを取得して返却
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    // 同じヘルパーを利用して、確認リンクを呼び出す追加のステップを追加
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // Postmarkに発行するリクエストがないことを想定
        .expect(0)
        .mount(&app.email_server)
        .await;

    // ニュースレターのペイロードの構造のスケッチ
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text</p>",
        "html_content": "Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));
    app.dispatch_all_pending_emails().await;
    // モックはドロップ時に、ニュースレターEメールを送信していないことを検証
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));
    app.dispatch_all_pending_emails().await;

    // モックがドロップされたとき、モックはニュースレターEメールを送信したことを検証
}

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_newsletter_form() {
    let app = spawn_app().await;

    let response = app.get_publish_newsletter().await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_publish_a_newsletter() {
    let app = spawn_app().await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // ニュースレターフォームを提出
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text.",
        "html_content": "<p>Newsletter body as HTML.</p>",
        // ヘッダーではなく、冪等キーをフォームデータの一部とする。
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // リダイレクトに従う
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));

    // *もう一度*、ニュースレターフォームを提出
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // リダイレクトに従う
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));

    app.dispatch_all_pending_emails().await;

    // ニュースレターメールを*1回だけ*送信したことを、モックはドロップした時に検証する。
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // 同時に2つのニュースレターフォームを投稿
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text.",
        "html_content": "<p>Newsletter body as HTML.</p>",
        "idempotency_key": Uuid::new_v4().to_string(),
    });
    let response1 = app.post_publish_newsletter(&newsletter_request_body);
    let response2 = app.post_publish_newsletter(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);

    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );

    app.dispatch_all_pending_emails().await;
}
