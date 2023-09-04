use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// 複数のレイヤをまとめて`tracing`のサブスクライバーを構成する 。
///
/// # 実装ノート
///
/// 返されるサブスクライバーの実際の型を記述することを避けるために、戻り値の型に複雑な`impl Subscriber`を使用する。
/// 返されるサブスクライバーは、後の`init_subscriber`に渡すことができるように、`Send`と`Sync`であるとを明示的に
/// 示す必要がある。
pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// スパンデータを処理するためにグローバルなデフォルトとしてサブスクライバーを登録する。
///
/// この関数は1回だけ呼び出されるべきである。
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
