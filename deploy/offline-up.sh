#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

IMAGE_FILE="${IMAGE_FILE:-easypaper-image.tar.gz}"
IMAGE_NAME_FILE="${IMAGE_NAME_FILE:-.easypaper-image-name}"

if [[ ! -f "$IMAGE_FILE" ]]; then
  echo "Missing $IMAGE_FILE. Please run this script from the unpacked offline package."
  exit 1
fi

if [[ -f "$IMAGE_NAME_FILE" ]]; then
  EASYPAPER_IMAGE="$(cat "$IMAGE_NAME_FILE")"
else
  EASYPAPER_IMAGE="${EASYPAPER_IMAGE:-easypaper:offline}"
fi

if [[ ! -f .env ]]; then
  cp .env.docker.example .env
  {
    echo ""
    echo "# Offline deployment image"
    echo "EASYPAPER_IMAGE=$EASYPAPER_IMAGE"
  } >> .env
  echo "Created .env from .env.docker.example."
  echo "Edit .env first, then run this script again."
  exit 1
fi

echo "Loading Docker image: $EASYPAPER_IMAGE"
gzip -dc "$IMAGE_FILE" | docker load

echo "Starting EasyPaper with image: $EASYPAPER_IMAGE"
EASYPAPER_IMAGE="$EASYPAPER_IMAGE" docker compose up -d

echo "Waiting for health check..."
for _ in $(seq 1 30); do
  if curl -fsS http://127.0.0.1:8787/api/health >/dev/null; then
    echo "EasyPaper is healthy: http://127.0.0.1:8787/api/health"
    exit 0
  fi
  sleep 1
done

echo "EasyPaper did not pass health check in time. Showing recent logs:"
docker compose logs --tail=80 easypaper
exit 1
