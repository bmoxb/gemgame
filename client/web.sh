#!/usr/bin/bash

TARGET_DIR="target/wasm32-unknown-unknown/release/"
readonly TARGET_DIR

set -e -x

cargo build --release --target wasm32-unknown-unknown
cp web/* $TARGET_DIR
cp -r assets/ $TARGET_DIR
cd $TARGET_DIR
wget https://not-fl3.github.io/miniquad-samples/gl.js https://not-fl3.github.io/miniquad-samples/sapp_jsutils.js
python -m http.server
