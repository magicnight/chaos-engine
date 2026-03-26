FROM rust:1.86-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconf
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY static/ static/
RUN cargo build --release

FROM alpine:3.20
RUN apk add --no-cache ca-certificates && mkdir -p /data/runs
COPY --from=builder /app/target/release/chaos /usr/local/bin/
VOLUME /data/runs
ENV DATABASE_PATH=/data/runs/chaos.db
EXPOSE 3117
HEALTHCHECK --interval=60s CMD wget -q --spider http://localhost:3117/api/v1/health || exit 1
ENTRYPOINT ["chaos"]
CMD ["serve", "--port", "3117"]
