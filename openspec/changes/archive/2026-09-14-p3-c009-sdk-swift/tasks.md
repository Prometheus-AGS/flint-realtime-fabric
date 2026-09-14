# Tasks — p3-c009 sdk-swift

- [ ] **T1** Create `sdks/swift/` directory structure
  - `mkdir -p sdks/swift/Sources/FrfClient`
  - Verification: directory exists

- [ ] **T2** Generate Swift bindings from frf-ffi
  - Build frf-ffi: `cargo build -p frf-ffi`
  - Run UniFFI bindgen:
    ```sh
    cargo run --bin uniffi-bindgen generate \
      --library target/debug/libfrf_ffi.dylib \
      --language swift \
      --out-dir sdks/swift/Sources/FrfClient
    ```
  - Verification: `sdks/swift/Sources/FrfClient/frf.swift` exists and is non-empty

- [ ] **T3** Create `sdks/swift/Package.swift`
  - Swift tools version: `5.9`
  - Package name: `FrfClient`
  - Platform: `.iOS(.v16)`, `.macOS(.v13)`
  - Products: `.library(name: "FrfClient", targets: ["FrfClient"])`
  - Targets: `.target(name: "FrfClient", path: "Sources/FrfClient", sources: ["frf.swift"])`
    (binary target for XCFramework added after framework build)
  - Verification: `swift package describe` exits 0 from `sdks/swift/`

- [ ] **T4** Create `sdks/swift/build_xcframework.sh`
  - Build `aarch64-apple-darwin` and `x86_64-apple-ios` slices via `cargo build --target ... -p frf-ffi`
  - Run `uniffi-bindgen` per slice for Swift
  - Run `xcodebuild -create-xcframework` to combine slices into `FrfClientFFI.xcframework`
  - Place XCFramework at `sdks/swift/Sources/FrfClient/FrfClientFFI.xcframework`
  - Verification: script is `chmod +x`; `./build_xcframework.sh --dry-run` prints steps without executing

- [ ] **T5** Create `sdks/swift/.gitignore`
  - Ignore: `*.a`, `*.dylib`, `*.o`, `FrfClientFFI.xcframework/`, `.build/`
  - But track: `Package.swift`, `build_xcframework.sh`, `Sources/FrfClient/frf.swift`
  - Verification: `git check-ignore -v sdks/swift/Sources/FrfClient/FrfClientFFI.xcframework` → ignored

- [ ] **T6** Verify Swift package compiles (host platform only — no simulator required)
  - `swift build` from `sdks/swift/` using host macOS target only
  - If XCFramework path not yet built, skip binary target temporarily to verify Swift source compiles
  - Verification: `swiftc -parse sdks/swift/Sources/FrfClient/frf.swift` exits 0 (parse-only check)

- [ ] **T7** Commit generated Swift bindings
  - Generated `frf.swift` is committed to the repo (it's generated from a frozen FFI surface)
  - `git add sdks/swift/` → commit
  - Verification: `git status sdks/swift/` clean after commit
