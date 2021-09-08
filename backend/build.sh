#!/usr/bin/env bash
set -euo pipefail

BACKEND_DIR="$(dirname "$0")"
TARGET="wasm32-unknown-unknown"

cargo build --manifest-path "$BACKEND_DIR/Cargo.toml" --target $TARGET --release -j1

cargo install ic-cdk-optimizer --version 0.3.1 --root "$BACKEND_DIR"/../target
STATUS=$?

if [ "$STATUS" -eq "0" ]; then
      "$BACKEND_DIR"/../target/bin/ic-cdk-optimizer \
      "$BACKEND_DIR/target/$TARGET/release/ic_butler.wasm" \
      -o "$BACKEND_DIR/target/$TARGET/release/ic_butler.wasm"

  true
else
  echo Could not install ic-cdk-optimizer.
  false
fi

