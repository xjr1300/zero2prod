use actix_web::HttpResponse;

/// ダミー実装
pub async fn publish_newsletter() -> HttpResponse {
    HttpResponse::Ok().finish()
}
