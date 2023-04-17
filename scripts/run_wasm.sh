#!/bin/bash

set -x
set -eo pipefail

static-web-server --port 8790 --root ./generated_wasm