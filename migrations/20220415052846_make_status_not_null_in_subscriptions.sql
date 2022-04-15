BEGIN;
-- 過去に登録された行のstatus列の値を'confirmed'に設定
UPDATE subscriptions
SET status = 'confirmed'
WHERE status IS NULL;
-- status列を必須に設定
ALTER TABLE subscriptions
ALTER COLUMN status
SET NOT NULL;
COMMIT;
