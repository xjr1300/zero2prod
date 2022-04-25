use std::fmt::Write;

use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn publish_newsletter_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<DOCTYPE html>
<html lang="ja">
<head>
    <meta http-equiv="content-type" content="text/html charset=utf-8">
    <title>ニュースレター発行</title>
</head>
<body>
    {msg_html}
    <form action="/admin/newsletter" method="post">
        <label>タイトル:
            <input type="text" placeholder="タイトルを入力" name="title" />
        </label>
        <br />
        <label>テキストコンテンツ
            <textarea placeholder="テキストコンテンツを入力" name="text_content" rows="20" cols="50"></textarea>
        </label>
        <br />
        <label>HTMLコンテンツ
            <textarea placeholder="HTMLコンテンツを入力" name="html_content" rows="20" cols="50"></textarea>
        </label>
        <br />
       <input type="submit" value="ニュースレター発行" />
    </form>
</body>
</html>"#,
        )))
}
