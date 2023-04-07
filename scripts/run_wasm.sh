#!/bin/bash

set -x
set -eo pipefail

static-web-server --port 8788 --root ./generated_wasm