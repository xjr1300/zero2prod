#!/usr/bin/env bash
set -x
set -eo pipefail

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

# 統合テスト用のデータベースを取得
export PGPASSWORD="${DB_PASSWORD}"
TEST_DBS=$(psql -h ${DB_HOST} -U ${DB_USER} -p ${DB_PORT} -c '\l' | grep test_db | cut -d "|" -f 1 | sed "s/^ *\| *$//")

# 統合テスト用のデータベースを削除
for TEST_DB in ${TEST_DBS[@]}; do
    psql -h ${DB_HOST} -U ${DB_USER} -p ${DB_PORT} -c "DROP DATABASE \"${TEST_DB}\""
done
