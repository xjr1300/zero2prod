# [Build]
# docker build --tag zero2prod --file Dockerfile .
#
# [Run]
# ポートの1番目の番号はローカルのポートを指定
# ポートの2番目の番号はコンテナのポートを指定
# この設定により、ローカルの8000番ポートにアクセスして、コンテナの8000番ポートにアクセス
# docker run -p 8000:8000 zero2prod

# 調理段階
FROM lukemathwalker/cargo-chef:latest-rust-1.72.0 as chef
WORKDIR /app
RUN apt update && apt install -y lld clang

# 計画段階
# 計画段階のコンテナはビルド段階のコンテナを無効にしないため、recipe.jsonが変更されない限り、
# ビルド段階のコンテナはキャッシュされる。
FROM chef as planner
COPY . .
# プロジェクトのロックファイルを処理
RUN cargo chef prepare --recipe-path recipe.json

# ビルド段階
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# アプリケーションではなく、プロジェクトの依存関係をビルド
RUN cargo chef cook --release --recipe-path recipe.json
# ここまできたら、もし依存関係のツリーが同じに止まっていれば、すべてのレイヤはキャッシュされる。
COPY . .
ENV SQLX_OFFLINE true
# プロジェクトをビルド
RUN cargo build --release --bin zero2prod

# 実行段階
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt update -y \
    && apt install -y --no-install-recommends openssl ca-certificates \
    # クリーンアップ
    && apt autoremove -y \
    && apt clean -y \
    &&  rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
