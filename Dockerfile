# Stage 1
FROM rust:latest as builder

WORKDIR /app
COPY . .
RUN cargo build --release


# Stage 2
FROM debian:bookworm-slim

RUN mkdir -p /app

RUN groupadd -g 999 appuser && \
    useradd -r -u 999 -g appuser appuser

USER appuser

COPY --from=builder /app/target/release/cas /app

WORKDIR /app

EXPOSE 8000

ENTRYPOINT ["./cas"]
