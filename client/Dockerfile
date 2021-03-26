FROM debian:buster-slim as builder

WORKDIR /usr/src/gemgame-client

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl wget build-essential \
    && apt-get clean

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && ~/.cargo/bin/rustup target add wasm32-unknown-unknown

COPY . .
RUN cd client/ && ~/.cargo/bin/cargo build --release --target wasm32-unknown-unknown

RUN cd client/web/public/ && wget https://not-fl3.github.io/miniquad-samples/gl.js \
    && wget https://not-fl3.github.io/miniquad-samples/sapp_jsutils.js --output-document jsutils.js

FROM jonasal/nginx-certbot:latest

COPY --from=builder /usr/src/gemgame-client/client/web/nginx.conf etc/nginx/conf.d/default.conf

WORKDIR /usr/share/nginx/html
COPY --from=builder /usr/src/gemgame-client/target/wasm32-unknown-unknown/release/gemgame-client.wasm client.wasm
COPY --from=builder /usr/src/gemgame-client/client/web/public/ .
COPY --from=builder /usr/src/gemgame-client/client/assets/ assets/
