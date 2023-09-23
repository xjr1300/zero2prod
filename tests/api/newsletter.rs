use std::time::Duration;

use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use uuid::Uuid;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, MockBuilder, ResponseTemplate};

use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
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
        .named("未確認の購読者を作成")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await.html;
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "ニュースレターのタイトル",
        "text_content": "テキストのニューズレターの本文",
        "html_content": "<p>HTMLのnニュースレタのn本文</p>",
        "idempotency_key": Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("ニュースレターの記事を発行しました。"));
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
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
        "title": "ニュースレターのタイトル",
        "text_content": "テキストのニューズレターの本文",
        "html_content": "<p>HTMLのnニュースレタのn本文</p>",
        "idempotency_key": Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("ニュースレターの記事を発行しました。"));
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
        "title": "ニュースレターのタイトル",
        "text_content": "テキストのニューズレターの本文",
        "html_content": "<p>HTMLのnニュースレタのn本文</p>",
        "idempotency_key": Uuid::new_v4().to_string(),
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

    let newsletter_request_body = serde_json::json!({
        "title": "ニュースレターのタイトル",
        "text_content": "テキストとしてのニューズレターの本文",
        "html_content": "HTMLとしてのニュースレターの本文",
        "idempotency_key": Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("ニュースレターの記事を発行しました。"));

    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("ニュースレターの記事を発行しました。"));
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

    let newsletter_request_body = serde_json::json!({
        "title": "ニュースレターのタイトル",
        "text_content": "テキストとしてのニューズレターの本文",
        "html_content": "HTMLとしてのニュースレターの本文",
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
}

fn when_sending_an_email() -> MockBuilder {
    Mock::given(path("/email")).and(method("POST"))
}

#[tokio::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retries() {
    let app = spawn_app().await;
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": Uuid::new_v4().to_string(),
    });
    create_confirmed_subscriber(&app).await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    // ニュースレターフォームの送信
    // 2番目のEメールの配信に失敗させる。
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .named("Delivery email to the second subscriber")
        .mount(&app.email_server)
        .await;

    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 500);

    // フォームの送信を再施行
    // 二人の購読者両方に対してのEメールの配信に。成功する。
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .named("Delivery retry")
        .mount(&app.email_server)
        .await;
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_eq!(303, response.status().as_u16());
}
