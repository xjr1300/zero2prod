-- ヘッダーの名前と値を表現する型を定義
CREATE TYPE header_pair AS (name TEXT, value BYTEA);
-- 送信したニュースレターを記録する表を定義
CREATE TABLE idempotency (
    user_id uuid NOT NULL REFERENCES users(user_id),
    idempotency_key TEXT NOT NULL,
    response_status_code SMALLINT NOT NULL,
    response_headers header_pair [] NOT NULL,
    response_body BYTEA NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (user_id, idempotency_key)
);
