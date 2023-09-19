use std::fmt::Write;

use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;

use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};

pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>パスワード変更</title>
</head>
<body>
    {msg_html}
    <form action="/admin/password" method="post">
        <label>
            現在のパスワード
            <input type="password" placeholder="現在のパスワードを入力" name="current_password" >
        </label>
        <br>
        <label>
            新しいパスワード
            <input type="password" placeholder="新しいパスワードを入力" name="new_password" >
        </label>
        <br>
        <label>
            新しいパスワードを確認
            <input type="password" placeholder="新しいパスワードを再入力" name="new_password_check" >
        </label>
        <br>
        <button type="submit">Change password</button>
    </form>
    <p><a href="/admin/dashboard">&lt;- 戻る</a></p>
</body>
</html>"#,
    )))
}
