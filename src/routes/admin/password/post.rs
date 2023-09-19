use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};

use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    session: TypedSession,
    form: web::Form<FormData>,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "あなたは異なる2つの新しいパスワードを入力しました - 2つのフィールドの値は一致する必要があります。",
        ).send();
        return Ok(see_other("/admin/password"));
    }
    todo!()
}
