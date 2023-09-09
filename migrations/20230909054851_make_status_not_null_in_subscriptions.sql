BEGIN;

-- 過去のエントリの`status`を`confirmed`で埋める。
UPDATE
    subscriptions
SET
    status = 'confirmed'
WHERE
    status IS NULL;

-- `status`を必須にする。
ALTER TABLE
    subscriptions
ALTER COLUMN
    status
SET
    NOT NULL;

COMMIT;
