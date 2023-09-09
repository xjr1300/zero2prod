#!/usr/bin/env bash
set -x
set -eo pipefail

# commandで指定されたコマンドが存在するか確認
if ![ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed"
    exit 1
fi
if ![ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed"
    echo >&2 "Use:"
    echo >&2 "    cargo install sqlx-cli --no-default-features --features rustls,postgres"
    echo >&2 "to install it"
    exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

# Docker化されたPostgresデータベースが、すでに起動している場合はスキップ
if [[ -z "${SKIP_DOCKER}" ]]; then
    docker run \
        --name zero2prod-db \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d postgres \
        postgres -N 1000
fi

# PostgreSQLがコマンドを受け付ける準備ができるまでピンを続ける
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    echo >&2 "Postgres is still unavailablle - sleeping"
    sleep 1
done

echo >&2 "Postgres is up and running on port ${DB_PORT}:!"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

echo >&2 "Postgres has been migrated, ready to go!"
