#!/usr/bin/env bash
# Regenerate the Dart/Flutter SDK from frf-ffi via flutter_rust_bridge_codegen 2.11.1.
#
# Prerequisites:
#   - flutter_rust_bridge_codegen 2.11.1: `cargo install flutter_rust_bridge_codegen`
#   - Flutter SDK on PATH
#   - Rust toolchain with frf-ffi compilable for the host
#
# Usage (from workspace root):
#   ./sdks/dart/build_dart.sh

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DART_SDK_DIR="$WORKSPACE_ROOT/sdks/dart"
FFI_CRATE="$WORKSPACE_ROOT/crates/frf-ffi"

echo "==> Generating Dart bridge from frf-ffi..."
flutter_rust_bridge_codegen generate \
    --rust-input "$FFI_CRATE/src/lib.rs" \
    --dart-output "$DART_SDK_DIR/lib/src/rust/frb_generated.dart" \
    --dart-root "$DART_SDK_DIR" \
    --rust-root "$WORKSPACE_ROOT"

echo "==> Running flutter pub get..."
(cd "$DART_SDK_DIR" && flutter pub get)

echo "==> Done. Generated files:"
find "$DART_SDK_DIR/lib/src/rust" -name "*.dart" | sort
