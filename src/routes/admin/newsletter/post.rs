use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

use crate::{
    authentication::UserId,
    domain::SubscriberEmail,
    email_client::EmailClient,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
}

pub async fn publish_newsletter(
    pool: web::Data<PgPool>,
    _user_id: web::ReqData<UserId>,
    email_client: web::Data<EmailClient>,
    form: web::Form<FormData>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscribers = get_confirmed_subscribers(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &form.0.title,
                        &form.0.html_content,
                        &form.0.text_content,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "ニュースレターの発行を{}に送信できませんでした。",
                            subscriber.email
                        )
                    })
                    .map_err(e500)?;
            }
            Err(e) => {
                tracing::warn!(
                    e.cause_chain = ?e,
                    e.message = %e,
                    "確認した購読者をスキップしました。蓄積された彼らの連絡先は不正です。"
                );
            }
        }
    }
    FlashMessage::info("ニュースレターの記事を発行しました。").send();

    Ok(see_other("/admin/newsletters"))
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> anyhow::Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(e) => Err(anyhow::anyhow!(e)),
    })
    .collect();

    Ok(confirmed_subscribers)
}
