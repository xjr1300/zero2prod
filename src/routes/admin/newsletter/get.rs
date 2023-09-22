use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use uuid::Uuid;

use crate::routes::admin::flash_messages_html;
use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};

pub async fn publish_newsletter_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let msg_html = flash_messages_html(flash_messages);
    let idempotency_key = Uuid::new_v4().to_string();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>ニュースレター発行</title>
</head>
<body>
    {msg_html}
    <form action="/admin/newsletter" method="post">
        <label>
            タイトル
            <input type="input" placeholder="タイトルを入力" name="title" >
        </label>
        <br>
        <label>
            本文（テキスト）
            <textarea name="text_content" placeholder="テキストコンテンツを入力" rows="10" cols="50"></textarea>
        </label>
        <br>
        <label>
            本文（HTML）
            <textarea name="html_content" placeholder="HTMLコンテンツを入力" rows="10" cols="50"></textarea>
        </label>
        <br>
        <input hidden type="text" name="idempotency_key" value="{idempotency_key}">
        <button type="submit">ニュースレターを発行</button>
    </form>
    <p><a href="/admin/dashboard">&lt;- 戻る</a></p>
</body>
</html>"#,
        )))
}
