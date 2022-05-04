# Zero To Production In Rust

## 1.4 内部開発ループ

ソフトウェアの開発は、`内部開発ループ`と呼ばれる以下に示すステップを繰り返し続ける。

1. 変更する。
2. アプリケーションをコンパイルする。
3. テストを実行する。
4. アプリケーションを実行する。

`内部開発ループ`のスピードは、単位時間で完了するいてらーションの数が上限になる。

### 1.4.1 高速リンキング

`Rust`でプログラムをコンパイルする際、リンキングフェーズに多くの時間が費やされる。
このため、内部開発ループの繰り返しを高速にするためにはリンカーが重要になる。
デフォルトのリンカーは適切に動作するが、使用するOSによって高速に処理する代替手段がある。

* `Windows`や`linux`では`lld`（`LLVM`プロジェクトが開発されたリンカー）
* `MacOS`では`zld`

リンキングフェーズの速度を上げるために、開発環境に合わせた互換のあるリンカーを導入して、プロジェクト用の設定ファイルを追加する必要がある。

```toml
# .cargo/config.toml
# On Windows
# ```
# cargo install -f cargo-binutils
# rustup component add llvm-tools-preview
# ```
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# On Linux:
# - Ubuntu, `sudo apt-get install lld clang`
# - Arch, `sudo pacman -S lld clang`
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "linker=clang", "-C", "link-arg=-fuse-ld=lld"]

# On MacOS, `brew install michaeleisel/zld/zld`
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/usr/local/bin/zld"]
[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/usr/local/bin/zld"]
```

`MacOS`では、以下のコマンドで`zld`をインストールできる。
ただし、`Xcode`(`Xcode command line tools`ではなく）を導入する必要がある。

[michaeleisel/zld](https://github.com/michaeleisel/zld)

また、`lld`よりも高速とされている新しい[mold](https://github.com/rui314/mold)がある。

## 1.4.2 `cargo-watch`

`cargo-watch`を使用すると、ソースコードが保存されるたびに、`cargo`コマンドを発行することができる。
`cargo-watch`のインストール方法を以下に示す。

```sh
cargo install cargo-watch
```

`cargo-watch`の利用方法を以下に示す。
なお、`cargo-watch`は、`fmt`や`clippy`など、`cargo`コマンドの連鎖をサポートする。

```sh
# ソースコードが保存されるたびに、checkコマンドを実行
cargo watch -x check
# ソースコードが保存されるたびに、fmt、clippy、checkコマンドを実行
cargo watch -x fmt -x clippy -x check
```

`cargo-watch`で自動的にソースコードを整形または検証することで、`内部開発ループ`を高速にできる。

## 1.5.1.2 コードカバレッジ

書籍では[tarpaulin](https://github.com/xd009642/tarpaulin)が紹介されているが、`tarpaulin`は、`x86_64`な`Linux`のみサポートしているため、`MacOS`では利用できない。

**TODO: `MacOS`で動作するコードカバレッジツールを調査すること。**

## 1.5.1.3 リント、1.5.1.4 フォーマット

ソースコードのリント及びフォーマットは、`Rust`と同時に導入される`clippy`と`rustfmt`を使用する。

```sh
# フォーマット
cargo fmt
# リント
cargo clippy
```

##　1.5.1.5 セキュリティの脆弱性

`cargo-audit`は、プロジェクトが依存するクレートと、クレートが依存するクレートを再帰的に検索して、それらクレートに脆弱性があるか確認する。
`cargo-audit`の導入方法と利用方法を以下に示す。

```sh
# 導入
cargo install cargo-audit
# 実行
cargo audit
```

## 1.5.2 `CI`パイプラインの準備

`CI`パイプラインで以下を実行する。

* 動作環境構築
* 脆弱性検査
* ソースコードのフォーマット
* ソースコードのリント
* テスト（単体テスト、統合テスト）
* （テストカバレッジ計測）

**TODO: `GitHub Actions`でCIパイプラインを実行する方法を調査すること。**

# 3.3.2.4 `tokio`ランタイム

`tokio`ランタイムを使用した、`actix-web`フレームワークを使用したアプリケーションのスケルトンを以下に示す。

```toml
# Cargo.toml
[package]
name = "actix-web-skeleton"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use actix_web::{web, App, HttpResponse, HttpServer};

async fn hello() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/hello", web::get().to(hello)))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
```

## 3.3.2.4 マクロの展開 - `tokio`ランタイム

上記、`#[tokio::main]`のようなマクロは、`cargo-expand`で展開できる。
なお、`cargo-expand`は`nightly`ビルドで動作する。

```sh
# `cargo-expand`の導入
cargo install cargo-expand
# `nightly`ビルドの導入
# `--allow-downgrade`で、必要なすべてのコンポーネントが利用可能な
# 最新の夜間を見つけてインストールするように指示
rustup toolchain install nightly --allow-downgrade
# マクロの展開
cargo +nightly expand
```

## 3.3.3 ヘルスチェックハンドラの実装

アプリが少なくとも応答することを確認するために、ヘルスチェック機能を実装する。
システム監視システムが、定期的にヘルスチェックを呼び出して、動作状況を確認する。

```rust
/// ヘルスチェックハンドラ
async fn health_check() -> HttpResponse {
     HttpResponse::Ok()
}
```

## 3.4.3 テストを容易にするためのプロジェクト構成

統合テストは、プロジェクトルートに作成した`tests`ディレクトリ内のファイルに実装する。
単体テストは、テスト対象が実装されているファイル内の`tests`モジュールに実装するか、ドキュメンテーションテストとして実装する。

統合テストは、他の統合テストの影響を受けないように、テストごとに以下の環境を構築及び削除して実行する。

* テスト用データベースなどの永続化層の構築と削除
* テスト用サーバー（`actix_web::App`）の構築と削除
* その他、連携するモックサーバーの構築と削除

なお、ドキュメンテーションテストは、`Rust for Rustaceans`を参照すること。

## 3.5 最初の統合テストの実装

統合テストでは、`HTTP`クライアントに`reqwest`を使用する。

```toml
# Cargo.toml
# [...]
#
[dev-dependencies]
reqwest = "0.11"
# [...]
```

## 3.5.1.2 テスト用サーバーが使用するポートの選択

テストサーバーは、使用されていないランダムなポートを選択するようにする。
`0`ポートの使用を試みると、`OS`は利用可能なポートにバインドする。
統合テストでは、ポート番号を指定してアプリのエンドポイントを呼び出すため、`OS`が選択したポートを以下の通り取得する。

```rust
/// テスト用のHTTPサーバーを起動して、テスト用アプリのURLを返却する。
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
```

## 3.8.2 データベースクレートの選択

`PostgreSQL`と相互作用するクレートは、2020年8月時点で以下などがある。

* `tokio-postgres`
* `sqlx`
* `diesel`

データベースクレートを以下3つの観点で検証する。

* コンパイルタイム安全性
* クエリインターフェース
* 非同期、同期インターフェース

コンパイルタイム安全性、非同期、`DSL`を学習する必要がない、`sqlx`を優先に検討する方が良いと考えられる。

## 3.8.5.2 設定管理

設定ファイルの読み込みは、[config](https://crates.io/crates/config)クレートを使用する。
`config`クレートは、`JSON`や`YAML`などのファイル形式を解析できる。

アプリの設定を、基本（`base`）、開発環境（`local`）、運用環境（`production`）別にファイルと環境変数に記録して、実行時に反映する。
アプリの設定の読み込みは、最初に`base`を読み込み、その後、環境にあわせて`local`または`production`を、次に環境変数を読み込んで、上書きする。

設定ファイルは、プロジェクトのルートに`configurations`ディレクトリなどを作成して、その中に環境別のファイルを配置する。
設定ファイルに秘匿事項を含める場合は、`ansible`などで暗号化しておきデプロイ時に複合化するか、ソース管理しないようにすること。

なお、`config`クレートは、読み込んだ設定を`config::Config`インスタンスに記録するが、それをアプリの設定を管理する構造体に`try_into`する。

## テレメトリー

ログを`tracing`クレートを使用して出力する。
また、`tracing_appender`クレートには、ログファイルをローテーションする機能がある。
ただし、`tracing_appender`クレートには、ログファイル数を制限する機能がないため、`rolling-file`クレートを合わせて使用する。
アプリ以外のログを出力する場合は、`tracing-log`クレートを使用する。

`tracing::span`を使用すると、スパンの開始と終了をログに出力する。
また、`tracing::instrument`マクロを使用すると、関数の開始と終了をログに出力する。

`tracing`でログを適切に取得するには、サブスクライバを設定すること。

## 4.5.13 secrecyによる秘密の保護

パスワードなどは、`secrecy::Secret`で保護する。

```rust
use secrecy::Secret;
// [...]
pub struct DatabaseSettings {
    // [...]
    pub password: Secret<String>,
    // [...]
}
```

## 6.5.1 AsRef

型が十分に`T`型に似ていて、型の参照から`T`の参照を得たい場合は、`AsRef`を実装する。

```rust
pub struct Username(String);

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
```

## 6.8 洞察を含んだエラーの表明: claim

`claim`クレートを使用すると、ビルトインされているアサートマクロよりも、多くの情報を提示してくれる。

## 6.13.1 `fake`でランダムなテストデータを生成する方法

`fake`クレートを使用すると、名前やEメールアドレスなどをランダムで生成して提供してくれる。

## 7.2.3.1 `wiremock`で`HTTP`サーバーをモックアップ

`wiremock::MockServer`で、`HTTP`サーバーをモックアップして、テストで利用する。
`wiremock::MockServer`は、何回りクエストを受診したかをカウントして、検証できる。

## 7.2.6.2 タイムアウト

経験則から言えば、いつでもIO操作を実行するときは、タイムアウトを設定する必要がある。
もしサーバーが応答するためにタイムアウトよりも長い時間を要する場合、失敗してエラーを返却するべきである。
適切なタイムアウト値を選択することは、科学というよりも芸術であり、特に再試行を伴う場合、とても小さいタイムアウト値を設定した場合、再試行リクエストを過剰に与えることになり、とても大きいタイムアウト値を設定した場合、クライアント側から見ると品質が劣化したとみられる危険性がある。
それも関わらず、タイムアウトを持たないよりも、保守的なタイムアウトの閾値を持つ方が良い。

サーバーがタイムアウトしたときのテストも必要である。

## 7.3.6 ひとつのテストファイルは、ひとつのクレート

ルートフォルダにある`tests`ディレクトリにある統合テスを実行するファイルは、それ自身のひとつのクレートにコンパイルされる。

