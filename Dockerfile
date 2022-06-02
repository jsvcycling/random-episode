FROM rust:1.60.0-alpine3.15 AS builder

WORKDIR /app

COPY . .

RUN apk add libc-dev openssl-dev && cargo install --path .

FROM alpine:3.15

COPY --from=builder /usr/local/cargo/bin/random-episode /usr/local/bin/random-episode

CMD ["/usr/local/bin/random-episode"]
