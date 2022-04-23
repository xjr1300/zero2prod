#!/usr/bin/env bash
set -x
set -eo pipefail

# redisコンテナが起動している場合は、停止して終了するための命令を出力
RUNNING_CONTAINER=$(docker ps --filter 'name=redis' --format '{{.ID}}')
if [[ -n $RUNNING_CONTAINER ]]; then
    echo >&2 "there is a redis container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_CONTAINER}"
    exit 1
fi

# dockerを使用してredisを起動
docker run \
    -p "6379:6379" \
    -d \
    --name "redis_$(date '+%s')" \
    redis:6

echo >&2 "redis is ready to go!"
