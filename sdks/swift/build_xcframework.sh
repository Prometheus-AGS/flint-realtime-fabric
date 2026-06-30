#!/usr/bin/env bash
# Build FrfClientFFI.xcframework from frf-ffi (UniFFI 0.31.2 proc-macro approach).
#
# Prerequisites:
#   - Rust toolchains: aarch64-apple-ios, aarch64-apple-darwin, x86_64-apple-ios
#   - cargo-swift or uniffi-bindgen in PATH (used for Swift bindings)
#   - Xcode command-line tools
#
# Usage:
#   cd <workspace-root>
#   ./sdks/swift/build_xcframework.sh
#
# Output:
#   sdks/swift/Sources/FrfClient/FrfClientFFI.xcframework
#   sdks/swift/Sources/FrfClient/frf.swift  (generated, do not hand-edit)

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SWIFT_SDK_DIR="$WORKSPACE_ROOT/sdks/swift"
SOURCES_DIR="$SWIFT_SDK_DIR/Sources/FrfClient"
BUILD_DIR="$WORKSPACE_ROOT/target"
XCFRAMEWORK_DIR="$SOURCES_DIR/FrfClientFFI.xcframework"

TARGETS=(
    "aarch64-apple-ios"
    "aarch64-apple-darwin"
    "x86_64-apple-ios"
)

echo "==> Building frf-ffi for all targets..."
for TARGET in "${TARGETS[@]}"; do
    echo "    cargo build --release -p frf-ffi --target $TARGET"
    cargo build --release -p frf-ffi --target "$TARGET"
done

echo "==> Generating Swift bindings via uniffi-bindgen..."
BINDGEN_OUT="$BUILD_DIR/uniffi-bindgen"
cargo run --bin uniffi-bindgen -- generate \
    --library "$BUILD_DIR/aarch64-apple-darwin/release/libfrf_ffi.dylib" \
    --language swift \
    --out-dir "$BINDGEN_OUT"

cp "$BINDGEN_OUT/frf.swift" "$SOURCES_DIR/frf.swift"
cp "$BINDGEN_OUT/frfFFI.h" "$SOURCES_DIR/frfFFI.h"
cp "$BINDGEN_OUT/frfFFI.modulemap" "$SOURCES_DIR/module.modulemap"

echo "==> Creating XCFramework slices..."
IOS_ARM64="$BUILD_DIR/aarch64-apple-ios/release/libfrf_ffi.a"
IOS_SIM_X86="$BUILD_DIR/x86_64-apple-ios/release/libfrf_ffi.a"
MACOS_ARM64="$BUILD_DIR/aarch64-apple-darwin/release/libfrf_ffi.a"

HEADERS_DIR="$BUILD_DIR/headers"
mkdir -p "$HEADERS_DIR"
cp "$SOURCES_DIR/frfFFI.h" "$HEADERS_DIR/"
cp "$SOURCES_DIR/module.modulemap" "$HEADERS_DIR/"

rm -rf "$XCFRAMEWORK_DIR"

xcodebuild -create-xcframework \
    -library "$IOS_ARM64" \
    -headers "$HEADERS_DIR" \
    -library "$IOS_SIM_X86" \
    -headers "$HEADERS_DIR" \
    -library "$MACOS_ARM64" \
    -headers "$HEADERS_DIR" \
    -output "$XCFRAMEWORK_DIR"

echo "==> Done: $XCFRAMEWORK_DIR"
