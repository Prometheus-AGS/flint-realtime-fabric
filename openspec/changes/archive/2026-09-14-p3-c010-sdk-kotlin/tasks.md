# Tasks — p3-c010 sdk-kotlin

- [ ] **T1** Create `sdks/kotlin/` directory structure
  - `mkdir -p sdks/kotlin/lib/src/main/kotlin/uniffi/frf`
  - `mkdir -p sdks/kotlin/lib/src/main/jniLibs`
  - Verification: directories exist

- [ ] **T2** Generate Kotlin bindings from frf-ffi
  - Build frf-ffi: `cargo build -p frf-ffi` (already built from p3-c008)
  - Run UniFFI bindgen:
    ```sh
    cargo run --bin uniffi-bindgen generate \
      --library target/debug/libfrf_ffi.dylib \
      --language kotlin \
      --out-dir sdks/kotlin/lib/src/main/kotlin
    ```
  - Verification: `sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt` exists and is non-empty

- [ ] **T3** Create `sdks/kotlin/settings.gradle.kts`
  - `rootProject.name = "frf-kotlin"`
  - `include("lib")`
  - Verification: file exists

- [ ] **T4** Create `sdks/kotlin/build.gradle.kts` (root)
  - `plugins { kotlin("jvm") version "2.0.0" apply false }`
  - Verification: file exists

- [ ] **T5** Create `sdks/kotlin/lib/build.gradle.kts`
  - `plugins { id("java-library"); kotlin("jvm") }`
  - `group = "ai.prometheusags.frf"; version = "0.1.0"`
  - Dependencies: `implementation("net.java.dev.jna:jna:5.14.0")` (required by UniFFI Kotlin runtime)
  - Verification: file exists

- [ ] **T6** Add Gradle wrapper
  - `gradle wrapper --gradle-version 8.7` from `sdks/kotlin/` — or manually create `gradle/wrapper/gradle-wrapper.properties` pointing to Gradle 8.7
  - Commit wrapper files (`gradlew`, `gradle/wrapper/`)
  - Verification: `./gradlew --version` from `sdks/kotlin/` exits 0 (downloads Gradle on first run)

- [ ] **T7** Create `sdks/kotlin/build_jni.sh`
  - Build `aarch64-apple-darwin` (macOS) and comment out Android targets (require NDK)
  - Copy `target/aarch64-apple-darwin/debug/libfrf_ffi.dylib` to `lib/src/main/jniLibs/libfrf_ffi.dylib`
  - Make executable
  - Verification: script exists; `./build_jni.sh --dry-run` exits 0

- [ ] **T8** Create `sdks/kotlin/.gitignore`
  - Ignore: `build/`, `.gradle/`, `lib/src/main/jniLibs/*.so`, `lib/src/main/jniLibs/*.dylib`
  - Track: `lib/src/main/kotlin/uniffi/frf/frf.kt`, `gradlew`, `gradle/`, all `*.kts`
  - Verification: `git check-ignore sdks/kotlin/build/` → ignored

- [ ] **T9** Compile-check Kotlin binding (without system kotlinc)
  - `./gradlew :lib:compileKotlin` from `sdks/kotlin/`
  - Note: requires JDK 17+ on PATH (check `java -version`); Gradle downloads Kotlin toolchain
  - Verification: `./gradlew :lib:compileKotlin` exits 0; `.class` files in `lib/build/classes/`
