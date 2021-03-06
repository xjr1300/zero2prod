use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;

// エラールートのログの原因を保持しながら、不透明な500を返します。
pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

// 本体の検証結果のユーザー表現とともに400を返却する。
// ロギングする目的から、根元で発生したエラールートを保存する。
pub fn e400<T: std::fmt::Debug + std::fmt::Display>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorBadRequest(e)
}
