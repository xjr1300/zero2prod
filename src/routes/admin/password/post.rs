use actix_web::error::InternalError;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::routes::admin::dashboard::get_username;
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
    pool: web::Data<PgPool>,
    form: web::Form<FormData>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = reject_anonymous_users(session).await?;

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "あなたは異なる2つの新しいパスワードを入力しました - 2つのフィールドの値は一致する必要があります。",
        ).send();
        return Ok(see_other("/admin/password"));
    }

    if !(12..=128).contains(&form.new_password.expose_secret().len()) {
        FlashMessage::error("パスワードは12文字以上128文字以下である必要があります。").send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(user_id, &pool).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("現在のパスワードが間違っています。").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e)),
        };
    }

    crate::authentication::change_password(user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::info("パスワードを変更しました。").send();

    Ok(see_other("/admin/password"))
}

async fn reject_anonymous_users(session: TypedSession) -> Result<Uuid, actix_web::Error> {
    match session.get_user_id().map_err(e500)? {
        Some(user_id) => Ok(user_id),
        None => {
            let response = see_other("/login");
            let e = anyhow::anyhow!("ユーザーはログインしていません。");
            Err(InternalError::from_response(e, response).into())
        }
    }
}
