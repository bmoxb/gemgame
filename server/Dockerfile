FROM rust:latest as builder

WORKDIR /usr/src/gemgame-server
COPY . .

RUN cargo install --path server/

FROM debian:buster-slim

RUN apt-get update && apt-get install -y --no-install-recommends libssl-dev && apt-get clean

COPY --from=builder /usr/local/cargo/bin/gemgame-server /usr/local/bin/gemgame-server

EXPOSE 5678

CMD gemgame-server --log-to-file --log-debug \
    --database-connection-string postgres://postgres:${DB_PASSWORD}@db/gemgame
