#--------------------------------------
#           Recipe stage
#--------------------------------------
FROM lukemathwalker/cargo-chef:latest-rust-1.60.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

#--------------------------------------
#           Planner stage
#--------------------------------------
FROM chef as planner
COPY . .
# ロックファイルを読み込んでレシピを作成
RUN cargo chef prepare --recipe-path recipe.json

#--------------------------------------
#           Builder stage
#--------------------------------------
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# プロジェクの依存関係を構築
RUN cargo chef cook --release --recipe-path recipe.json
# この時点で、もし依存関係ツリーが同じであれば、すべてのレイヤはキャッシュされる。
# 作業環境からすべてのファイルをイメージ(/app)にコピー
COPY . .
# sqlxがクエリのメタデータをファイルから取得するようにフラグを設定
ENV SQLX_OFFLINE true
# バイナリを構築
RUN cargo build --release --bin zero2prod

#--------------------------------------
#           Runtime stage
#--------------------------------------
FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# コンパイルされたバイナリをビルダー環境からランタイム環境にコピー
COPY --from=builder /app/target/release/zero2prod zero2prod
# 設定ファイルをランタイム環境にコピー
COPY configuration configuration
# プロダクション環境であることを指定
ENV APP_ENVIRONMENT production
# `docker run`が実行されたとき、バイナリを実行
ENTRYPOINT ["./target/release/zero2prod"]
