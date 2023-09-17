use std::fmt::Write;

use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::{IncomingFlashMessages, Level};

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>ログイン</title>
</head>
<body>
    {error_html}
    <form action="/login" method="post">
        <label>
            ユーザー名
            <input type="text" name="username" placeholder="ユーザー名を入力" name="username">
        </label>
        <label>
            パスワード
            <input type="password" placeholder="パスワードを入力" name="password">
        </label>
        <button type="submit">ログイン</button>
    </form>
</body>
</html>"#,
        ))
}
