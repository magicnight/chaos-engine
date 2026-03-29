FROM rust:1.86-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconf

WORKDIR /app

# Cache dependencies in a separate layer
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release && rm -rf src

# Build actual source
COPY src/ src/
COPY static/ static/
RUN touch src/main.rs && cargo build --release

FROM alpine:3.20
RUN apk add --no-cache ca-certificates && \
    adduser -D chaos && \
    mkdir -p /data/runs && chown chaos:chaos /data/runs
COPY --from=builder /app/target/release/chaos /usr/local/bin/
USER chaos
VOLUME /data/runs
ENV DATABASE_PATH=/data/runs/chaos.db
EXPOSE 3117
HEALTHCHECK --interval=60s CMD wget -q --spider http://localhost:3117/api/v1/health || exit 1
ENTRYPOINT ["chaos"]
CMD ["serve", "--port", "3117"]
