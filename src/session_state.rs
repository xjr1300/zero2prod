use std::future::{ready, Ready};

use actix_session::{Session, SessionExt};
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use uuid::Uuid;

pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), serde_json::Error> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }

    pub fn get_user_id(&self) -> Result<Option<Uuid>, serde_json::Error> {
        self.0.get(Self::USER_ID_KEY)
    }
}

impl FromRequest for TypedSession {
    // Sessionが実装するFromRequestによって返却される同じ型のエラーをエラーとして定義
    type Error = <Session as FromRequest>::Error;
    // Rustはトレイトにおいて`async`構文をサポートしていない。
    // FromRequestはHTTP呼び出しなど非同期操作を実行するために、戻り値の型としてFutureを想定している。
    // I/O動作がないためFutureを持たないため、TypedSessionをReadyでラップして、実行者によって
    // 最初にポーリングされたときにラップされた値を解決するFutureに変換する。
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
