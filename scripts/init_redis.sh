#!/usr/bin/env bash
set -x
set -eo pipefail

# rediskコンテナが起動している、場合、それをs停止して終了する命令をプリント
CONTAINER_NAME="zero2prod-redis"
RUNNING_CONTAINERS=$(docker ps --filter "name=${CONTAINER_NAME}" --format '{{.ID}}')
if [[ -n $RUNNING_CONTAINER ]]; then
    echo >&2 "there is a redis container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_CONTAINER}"
    exit 1
fi

# Dockerを使用してRedisを起動
docker run -p "6379:6379" -d --name "${CONTAINER_NAME}" redis:7
echo >&2 "Redis is ready to go!"
