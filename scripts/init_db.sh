#!/usr/bin/env bash
set -x
set -eo pipefail

# Check if a custom parameter has been set, else use default values
DB_PORT="${POSTGRES_PORT:=5432}"
SUPERUSER="${SUPERUSER:=postgres}"
SUPERUSER_PWD="${SUPERUSER_PWD:=password}"

# Launch postress using Docker
CONTAINER_NAME="postgres"
docker run \
       --env POSTGRES_USER=${SUPERUSER} \
       --env POSTGRES_PASSWORD=${SUPERUSER_PWD} \
       --publish "${DB_PORT}":5432 \
       --detach \
       --name "${CONTAINER_NAME}" \
       postgres -N 1000
                  # ^ Increased maximum number of connections for testing purposes
