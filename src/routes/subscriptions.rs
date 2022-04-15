use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(data: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(&data.email)?;
        let name = SubscriberName::parse(&data.name)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Add a new subscriber",
    skip(pool, form, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(
    pool: web::Data<PgPool>,
    form: web::Form<FormData>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        // let new_subscriber = match NewSubscriber::try_from(form.0) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    // （使い物にならない）Eメールを新しい購読者に送信
    // 現段階では、Eメールの送信エラーを無視
    if email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            "Welcome to out newsletter!",
            "Welcome to out newsletter!",
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, new_subscriber)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
