FROM rust:latest as builder

WORKDIR /usr/src/gemgame-server
COPY . .

RUN cargo install --path server/

FROM debian:buster-slim

COPY --from=builder /usr/local/cargo/bin/gemgame-server /usr/local/bin/gemgame-server

EXPOSE 5678

CMD ["gemgame-server"]