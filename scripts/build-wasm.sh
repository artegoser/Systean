#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if command -v rustup >/dev/null 2>&1; then
    rustup target add wasm32-unknown-unknown >/dev/null
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "wasm-pack not found; installing wasm-pack 0.15.0..." >&2
    cargo install wasm-pack --version 0.15.0 --locked
fi

cd "$ROOT/crates/systean-wasm"
wasm-pack build . \
    --target web \
    --out-dir ../../site/src/lib/wasm/pkg \
    --release \
    --no-opt
