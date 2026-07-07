#!/usr/bin/env bash
# Regenerate the Dart/Flutter SDK from the UniFFI FFI crate (frf-ffi).
#
# The FFI crate is UniFFI-based (same surface as the Swift/Kotlin bindings), so
# the Dart bindings are generated with uniffi-bindgen-dart — NOT flutter_rust_bridge.
# FRB cannot parse a UniFFI crate (it errors on the #[uniffi::export] surface).
#
# Prerequisites:
#   - uniffi-bindgen-dart:  `cargo install uniffi-bindgen-dart`
#   - Dart/Flutter SDK on PATH
#   - Rust toolchain able to build frf-ffi as a cdylib for the host
#
# Usage (from workspace root):
#   ./sdks/dart/build_dart.sh

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DART_SDK_DIR="$WORKSPACE_ROOT/sdks/dart"
RUST_OUT="$DART_SDK_DIR/lib/src/rust"

echo "==> Building frf-ffi cdylib (release)..."
(cd "$WORKSPACE_ROOT" && cargo build -p frf-ffi --release)

LIB="$WORKSPACE_ROOT/target/release/libfrf_ffi.dylib"
[ -f "$LIB" ] || LIB="$WORKSPACE_ROOT/target/release/libfrf_ffi.so"

echo "==> Generating Dart bindings via uniffi-bindgen-dart..."
uniffi-bindgen-dart generate --library "$LIB" --out-dir "$RUST_OUT"

echo "==> Running dart pub get..."
(cd "$DART_SDK_DIR" && dart pub get)

echo "==> Done. Generated files:"
find "$RUST_OUT" -name "*.dart" | sort
