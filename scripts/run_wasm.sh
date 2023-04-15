#!/bin/bash

set -x
set -eo pipefail

PORT=8796

echo http://localhost:$PORT/index.html
static-web-server --port $PORT --root ./generated_wasm