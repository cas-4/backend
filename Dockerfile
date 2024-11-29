# Stage 1
FROM rust:latest AS builder

WORKDIR /app
COPY . .
RUN cargo build --release


# Stage 2
FROM debian:bookworm-slim

LABEL version="0.8.10"

RUN mkdir -p /app

RUN apt-get update && apt-get install -y libssl-dev ca-certificates
RUN groupadd -g 999 appuser && \
    useradd -r -u 999 -g appuser appuser

RUN mkdir -p /app/assets
RUN chown -R appuser:appuser /app/assets

USER appuser

COPY --from=builder /app/target/release/cas /app

WORKDIR /app

EXPOSE 8000

ENTRYPOINT ["./cas"]
