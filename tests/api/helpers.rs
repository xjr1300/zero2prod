use std::net::TcpListener;
use uuid::Uuid;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};

use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::EmailClient;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// once_cellを使用して、トレーシングスタックが1回しか初期化されないことを確実にする。
static TRACING: Lazy<()> = Lazy::new(|| {
    let name = "test".to_string();
    let level = "info".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(name, level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(name, level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // テスト用データベース名を設定
    configuration.database.database_name = Uuid::new_v4().to_string();
    let pool = configure_database(&configuration.database).await;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        &configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let server = zero2prod::startup::run(listener, pool.clone(), email_client)
        .expect("Failed to bind address.");
    let _ = tokio::spawn(server);
    TestApp { address, pool }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // データベース名を指定しないことで、template1データベースに接続
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Fail to connect to postgres.");
    // テスト用データベースを構築
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create test database.");

    // テスト用データベースに接続して、マイグレーションを実行
    let pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to test database.");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the test database.");
    pool
}
