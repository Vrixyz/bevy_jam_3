#!/bin/bash

set -x
set -eo pipefail

BINARY_NAME=Sidle_Ffect

cargo build --bin $BINARY_NAME --profile profile --target wasm32-unknown-unknown
cp -r wasm/* generated_wasm/
wasm-bindgen --no-typescript --out-name bevy_game --out-dir generated_wasm --target web ./target/wasm32-unknown-unknown/profile/$BINARY_NAME.wasm
cp -r assets generated_wasm/