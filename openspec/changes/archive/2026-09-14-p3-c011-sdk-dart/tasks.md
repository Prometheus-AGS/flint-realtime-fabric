# Tasks — p3-c011 sdk-dart

- [ ] **T1** Annotate `frf-ffi` public surface with `#[frb]` for flutter_rust_bridge
  - Add `flutter_rust_bridge = { version = "2.11.1" }` to `crates/frf-ffi/Cargo.toml`
  - Add `#[flutter_rust_bridge::frb(sync)]` or `#[flutter_rust_bridge::frb(opaque)]` annotations to `FrfClient` and `frf_version`
  - Verify: `cargo check -p frf-ffi` exits 0 with frb dependency

- [ ] **T2** Create `sdks/dart/` directory structure
  - `mkdir -p sdks/dart/lib/src/rust sdks/dart/lib`
  - Verification: directories exist

- [ ] **T3** Create `sdks/dart/pubspec.yaml`
  - Package name: `frf_dart`
  - Dependencies: `flutter_rust_bridge: ^2.11.1`, `ffi: ^2.1.0`
  - Verification: `flutter pub get` from `sdks/dart/` exits 0

- [ ] **T4** Run `flutter_rust_bridge_codegen` to generate Dart bridge
  - From workspace root:
    ```sh
    flutter_rust_bridge_codegen generate \
      --rust-input crates/frf-ffi/src/lib.rs \
      --dart-output sdks/dart/lib/src/rust
    ```
  - Verification: `sdks/dart/lib/src/rust/frb_generated.dart` exists and is non-empty

- [ ] **T5** Create `sdks/dart/lib/frf_dart.dart`
  - `export 'src/rust/frb_generated.dart';`
  - Verification: file exists

- [ ] **T6** Create `sdks/dart/build_dart.sh`
  - Runs `flutter_rust_bridge_codegen generate ...` (same as T4)
  - Runs `flutter pub get` from `sdks/dart/`
  - Make executable
  - Verification: script exists and is executable

- [ ] **T7** Create `sdks/dart/.gitignore`
  - Ignore: `.dart_tool/`, `build/`
  - Track: `lib/src/rust/frb_generated.dart`, `pubspec.yaml`, `pubspec.lock`, `lib/frf_dart.dart`
  - Verification: `git check-ignore sdks/dart/.dart_tool/` → ignored

- [ ] **T8** Verify Dart package analyzes cleanly
  - `flutter analyze` from `sdks/dart/` → 0 errors
  - Or `dart analyze lib/` if not running in a Flutter context
  - Verification: exits 0 with 0 errors (warnings from generated code acceptable)
