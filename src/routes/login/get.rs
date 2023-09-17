use actix_web::cookie::Cookie;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse};

pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()),
        None => "".into(),
    };

    let mut response = HttpResponse::Ok()
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
        ));

    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();

    response
}
