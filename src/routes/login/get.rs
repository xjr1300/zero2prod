use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(query: web::Query<QueryParams>) -> HttpResponse {
    let error_html = match query.0.error {
        Some(error_message) => format!("<p><i>{error_message}</i></p>"),
        None => "".into(),
    };

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
