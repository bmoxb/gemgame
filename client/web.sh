#!/usr/bin/bash

TARGET_DIR="target/wasm32-unknown-unknown/release/"
readonly TARGET_DIR

set -e -x

cargo build --release --target wasm32-unknown-unknown
cp web/* $TARGET_DIR
cp -r assets/ $TARGET_DIR
cd $TARGET_DIR
python -m http.server
