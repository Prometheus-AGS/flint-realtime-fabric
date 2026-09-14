# p3-c009 — UniFFI-generated Swift SDK + XCFramework build script

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c008 (frf-ffi crate built and bindgen verified)

## Directory
`sdks/swift/`

## What this change does

Generates the Swift SDK from `frf-ffi` using UniFFI 0.31.2, adds a build script
that produces an XCFramework for iOS + macOS, and scaffolds a minimal Swift
package manifest so the SDK is consumable via Swift Package Manager.

### Generated files (not hand-edited)

`sdks/swift/Sources/FrfClient/`:
- `frf.swift` — UniFFI-generated Swift bindings
- `frf.modulemap` — module map
- `FrfClientFFI.xcframework/` — built by `build_xcframework.sh`

### Hand-authored files

- `sdks/swift/Package.swift` — SPM manifest declaring `FrfClient` library target
- `sdks/swift/build_xcframework.sh` — invokes `cargo build`, `uniffi-bindgen`, and
  `xcodebuild -create-xcframework` for `arm64-apple-ios`, `arm64-apple-darwin`, and
  `x86_64-apple-ios` slices
- `sdks/swift/.gitignore` — ignores build artifacts, `*.a`, `*.dylib` intermediates

### Swift Package Manager target

```swift
// Package.swift
targets: [
    .binaryTarget(
        name: "FrfClientFFI",
        path: "Sources/FrfClient/FrfClientFFI.xcframework"
    ),
    .target(
        name: "FrfClient",
        dependencies: [.target(name: "FrfClientFFI")],
        path: "Sources/FrfClient",
        sources: ["frf.swift"]
    )
]
```

## Note on loro-swift

`loro-swift` v1.13.2 (June 2026) is an SPM package maintained by loro-dev that
wraps the Loro Rust core. FRF's Swift SDK wraps `frf-ffi` (our UniFFI scaffold),
not `loro-swift` directly — CRDT logic crosses the boundary as opaque `[UInt8]`
deltas. No dependency on `loro-swift` is needed in `sdks/swift/`.

## Non-goals

- Does not publish to Swift Package Index.
- Does not add iOS UI demo app (out of scope for Phase 3).
- Does not sign the XCFramework (CI pipeline handles signing in Phase 7).
- Does not depend on `loro-swift` — Loro is encapsulated inside `frf-ffi` (Rust-side).
