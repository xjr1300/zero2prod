# ベースイメージとしてRustの安定版リリースを使用
FROM rust:1.60.0

# 存在しなければappディレクトリを作成して、appディレクトリに移動
WORKDIR /app

# リンクを構成するために必要とされているシステム依存をインストール
RUN apt update && apt install lld clang -y

# 作業環境からすべてのファイルをイメージ(/app)にコピー
COPY . .

# sqlxがクエリのメタデータをファイルから取得するようにフラグを設定
ENV SQLX_OFFLINE true

# バイナリを構築
RUN cargo build --release

# プロダクション環境であることを指定
ENV APP_ENVIRONMENT production

# `docker run`が実行されたとき、バイナリを実行
ENTRYPOINT ["./target/release/zero2prod"]
