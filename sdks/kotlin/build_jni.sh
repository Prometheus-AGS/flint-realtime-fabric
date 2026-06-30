#!/usr/bin/env bash
# Build frf-ffi JNI dylib and generate Kotlin bindings via UniFFI 0.31.2.
#
# Prerequisites:
#   - Rust toolchain for the host (aarch64-apple-darwin or x86_64-unknown-linux-gnu)
#   - cargo run --bin uniffi-bindgen accessible in the workspace
#
# Usage (from workspace root):
#   ./sdks/kotlin/build_jni.sh
#
# Output:
#   sdks/kotlin/lib/src/main/jniLibs/libfrf_ffi.{dylib,so}
#   sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KOTLIN_SDK_DIR="$WORKSPACE_ROOT/sdks/kotlin"
JNILIB_DIR="$KOTLIN_SDK_DIR/lib/src/main/jniLibs"
KOTLIN_OUT="$KOTLIN_SDK_DIR/lib/src/main/kotlin/uniffi/frf"
BUILD_DIR="$WORKSPACE_ROOT/target"

# Determine host triple
HOST_TRIPLE="$(rustc --print host-tuple 2>/dev/null || rustc -vV | awk '/host:/{print $2}')"
echo "==> Host triple: $HOST_TRIPLE"

echo "==> Building frf-ffi (release) for $HOST_TRIPLE..."
cargo build --release -p frf-ffi

# Locate built library
if [[ "$OSTYPE" == "darwin"* ]]; then
    LIB_EXT="dylib"
else
    LIB_EXT="so"
fi
LIB_SRC="$BUILD_DIR/release/libfrf_ffi.$LIB_EXT"
LIB_DST="$JNILIB_DIR/libfrf_ffi.$LIB_EXT"

echo "==> Copying $LIB_SRC → $LIB_DST"
cp "$LIB_SRC" "$LIB_DST"

echo "==> Generating Kotlin bindings via uniffi-bindgen..."
cargo run --bin uniffi-bindgen -- generate \
    --library "$LIB_SRC" \
    --language kotlin \
    --out-dir "$KOTLIN_OUT"

echo "==> Done."
echo "    JNI library: $LIB_DST"
echo "    Kotlin binding: $KOTLIN_OUT/frf.kt"
