# [Build]
# docker build --tag zero2prod --file Dockerfile .
#
# [Run]
# ポートの1番目の番号はローカルのポートを指定
# ポートの2番目の番号はコンテナのポートを指定
# この設定により、ローカルの8000番ポートにアクセスして、コンテナの8000番ポートにアクセス
# docker run -p 8000:8000 zero2prod

# 私たちは基本イメージにRustの最新の安定化バージョンを使用する。
FROM rust:1.72.0

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

# `docker run`が実行されたとき、バイナリが起動する。
ENTRYPOINT ["./target/release/zero2prod"]
