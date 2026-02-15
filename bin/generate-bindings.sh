#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <python>"
  exit 1
fi

TARGET="$1"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FFI_CRATE_DIR="$ROOT_DIR/crates/json-joy-ffi"
TARGET_DIR="$ROOT_DIR/target/debug"

if [[ "$TARGET" != "python" ]]; then
  echo "unsupported target: $TARGET"
  exit 1
fi

LANG="python"
OUT_DIR="$ROOT_DIR/bindings/python/src/json_joy_rs/generated"

mkdir -p "$OUT_DIR"

# Build FFI crate first so bindgen can inspect symbols.
mise x -- cargo build -p json-joy-ffi

case "$(uname -s)" in
  Darwin)
    LIB_PATH="$TARGET_DIR/libjson_joy_ffi.dylib"
    ;;
  Linux)
    LIB_PATH="$TARGET_DIR/libjson_joy_ffi.so"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    LIB_PATH="$TARGET_DIR/json_joy_ffi.dll"
    ;;
  *)
    echo "unsupported OS: $(uname -s)"
    exit 1
    ;;
esac

if [[ ! -f "$LIB_PATH" ]]; then
  echo "ffi library not found at: $LIB_PATH"
  exit 1
fi

mise x -- cargo run -p embedded-uniffi-bindgen -- generate \
  --library "$LIB_PATH" \
  --language "$LANG" \
  --config "$FFI_CRATE_DIR/uniffi.toml" \
  --out-dir "$OUT_DIR"
