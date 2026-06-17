FROM node:22-bookworm-slim AS frontend-builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY index.html postcss.config.js tailwind.config.js tsconfig.json vite.config.ts ./
COPY src ./src
RUN npm run build


FROM rust:1.95-bookworm AS backend-builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p backend/src \
    && printf 'fn main() {}\n' > backend/src/main.rs \
    && printf '' > backend/src/lib.rs
RUN cargo fetch --locked

COPY backend/src backend/src
RUN cargo build --release --locked -p easypaper-backend


FROM debian:bookworm-slim AS runtime

ARG VCS_REF=""
ARG BUILD_DATE=""

LABEL org.opencontainers.image.title="EasyPaper" \
      org.opencontainers.image.description="Feynman-style interactive paper learning web app" \
      org.opencontainers.image.source="https://github.com/aaronlou/EasyPaper" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.created="${BUILD_DATE}"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /app/target/release/easypaper-backend /usr/local/bin/easypaper-backend
COPY --from=frontend-builder /app/dist /app/dist

RUN useradd --system --uid 10001 --home-dir /app easypaper \
    && mkdir -p /data/uploads \
    && chown -R easypaper:easypaper /app /data

ENV EASYPAPER_BIND_ADDR=0.0.0.0:8787 \
    EASYPAPER_DB_PATH=/data/easypaper.db \
    EASYPAPER_UPLOAD_DIR=/data/uploads \
    STATIC_DIR=/app/dist \
    RUST_LOG=easypaper_backend=info,tower_http=info

EXPOSE 8787

USER easypaper

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8787/api/health || exit 1

CMD ["easypaper-backend"]
