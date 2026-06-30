# p10-c004 — Kotlin SDK Gradle Build Verification

## Phase
phase-10-e2e-layer2-wasm-federation

## Summary

Format the generated `frf.kt` with ktlint, copy the native library to the
expected JNI path, and verify `./gradlew :lib:build` passes in `sdks/kotlin/`.

## Files to Create/Modify

- `sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt` — reformatted by ktlint
- `sdks/kotlin/lib/src/main/jniLibs/linux-x86-64/libfrf_ffi.so` — symlink or
  copy of `target/release/libfrf_ffi.so` (created at Gradle build time; not
  committed — added to `.gitignore`)
- `sdks/kotlin/.gitignore` — ignore `src/main/jniLibs/**/*.so`
- `sdks/kotlin/lib/build.gradle.kts` — add `ktlint` plugin or manual
  `checkstyle`-equivalent formatting step

## Design Notes

ktlint does not need to be a Gradle plugin — for a generated file it's simpler
to run `ktlint --format` as a post-generation step in Dagger Stage 3. The
formatted file is what gets committed.

For the Gradle build:
1. The `target/release/libfrf_ffi.so` must exist (requires Rust release build
   on Linux; macOS produces `libfrf_ffi.dylib`). Dagger Stage 3 runs on
   `rust:1.85-slim` (Linux) so the `.so` is available there.
2. `build.gradle.kts` already has `net.java.dev.jna:jna:5.14.0` as a compile
   dependency. The Gradle test compile requires the source to parse; the
   runtime JNI load is optional for a compile-only verification.
3. Add `tasks.withType<Test> { enabled = false }` in `lib/build.gradle.kts`
   to skip runtime JNI tests that need the `.so` present at test time.

## Exit Criteria

- `ktlint sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt` exits 0 after
  reformatting (no lint errors)
- `./gradlew :lib:compileKotlin` exits 0 from `sdks/kotlin/`
- No Kotlin compilation errors
- `sdks/kotlin/.gitignore` ignores the JNI native library files
