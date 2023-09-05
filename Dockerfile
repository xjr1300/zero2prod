# [Build]
# docker build --tag zero2prod --file Dockerfile .
#
# [Run]
# ポートの1番目の番号はローカルのポートを指定
# ポートの2番目の番号はコンテナのポートを指定
# この設定により、ローカルの8000番ポートにアクセスして、コンテナの8000番ポートにアクセス
# docker run -p 8000:8000 zero2prod

# ビルダー段階（一時的な中間イメージ）
# 私たちは基本イメージにRustの最新の安定化バージョンを使用する。
FROM rust:1.72.0 AS builder
# 作業ディレクトリを`app`に切り替える（`cd app`と同等）。
# `app`ディレクトリは、それが存在しない場合に、Dockerによって作成される。
WORKDIR /app
# 設定に関係する必要とされるシステム依存をインストールする。
RUN apt update && apt install lld clang -y
# 作業環境からDockerイメージへ全てのファイルをコピーする。
COPY . .
# バイナリをビルドする。
# アプリケーションの動作速度を向上させるために、リリースプロファイルを使用する。
ENV SQLX_OFFLINE true
RUN cargo build --release

# 実行段階（最終的なイメージ）
FROM debian:bookworm-slim as runtime
# 作業ディレクトリを`app`に切り替える（`cd app`と同等）。
WORKDIR /app
# OpenSSLをインストール: 私たちの依存関係のいくつかによって直接リンクされている。
# ca-certificatesをインストール: HTTP接続を確立するとき、TLS証明書を検証するために必要とされる。
RUN apt -y update \
    && apt -y install --no-install-recommends openssl ca-certificates \
    # クリーンアップ
    && apt -y autoremove \
    && apt -y clean \
    && rm -rf /var/lib/apt/lists/*
# ビルダー環境から実行環境にコンパイルしたバイナリをコピー
COPY --from=builder /app/target/release/zero2prod zero2prod
# 実行段階では設定ファイルが必要
COPY configuration configuration
# `docker run`が実行されたとき、バイナリが起動する。
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
