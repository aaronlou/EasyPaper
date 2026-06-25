#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

IMAGE="${EASYPAPER_IMAGE:-easypaper:offline}"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="${1:-release/easypaper-offline-$STAMP}"
PACKAGE="${OUT_DIR}.tar.gz"

echo "Building Docker image: $IMAGE"
docker build -t "$IMAGE" .

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR/deploy"

echo "Saving Docker image..."
docker save "$IMAGE" | gzip -c > "$OUT_DIR/easypaper-image.tar.gz"
printf '%s' "$IMAGE" > "$OUT_DIR/.easypaper-image-name"

cp compose.yaml "$OUT_DIR/compose.yaml"
cp .env.docker.example "$OUT_DIR/.env.docker.example"
cp deploy/Caddyfile.example "$OUT_DIR/deploy/Caddyfile.example"
cp deploy/offline-up.sh "$OUT_DIR/deploy/offline-up.sh"

tar -czf "$PACKAGE" -C "$(dirname "$OUT_DIR")" "$(basename "$OUT_DIR")"

echo ""
echo "Offline package created:"
echo "  $PACKAGE"
echo ""
echo "Upload it to your server, then run:"
echo "  mkdir -p /opt/easypaper"
echo "  tar -xzf $(basename "$PACKAGE") -C /opt/easypaper --strip-components=1"
echo "  cd /opt/easypaper"
echo "  bash deploy/offline-up.sh"
