# p3-c010 — UniFFI-generated Kotlin SDK + Gradle wrapper

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c008 (frf-ffi crate built and bindgen verified)

## Directory
`sdks/kotlin/`

## What this change does

Generates the Kotlin SDK from `frf-ffi` using UniFFI 0.31.2, and scaffolds a
Gradle project so the SDK is consumable as a library from Android or JVM projects.

Java consumers use this Kotlin binding directly — no separate Java SDK is needed.

### Generated files (not hand-edited)

`sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt` — UniFFI-generated Kotlin binding

### Hand-authored files

- `sdks/kotlin/build.gradle.kts` — Gradle build with `java-library` plugin
- `sdks/kotlin/settings.gradle.kts` — project name
- `sdks/kotlin/gradlew` + `gradle/wrapper/` — Gradle wrapper (6.8+ compatible)
- `sdks/kotlin/lib/src/main/jniLibs/` — native `.so` / `.dylib` populated by build script
- `sdks/kotlin/build_jni.sh` — builds `frf-ffi` Rust JNI dylib and copies to `jniLibs/`
- `sdks/kotlin/.gitignore` — ignores build artifacts

### Gradle artifact

- Group: `ai.prometheusags.frf`
- Artifact: `frf-kotlin`
- Exports `FrfClient`, `FrfError`, `EventCallback` from the generated binding

### kotlinc note

`kotlinc` is not on the developer machine's PATH. The Gradle wrapper (`./gradlew`)
resolves Kotlin from the Gradle toolchain — no system `kotlinc` required. CI uses
a container with `openjdk:17` + `gradle`.

## Non-goals

- Does not publish to Maven Central.
- Does not add Android demo app.
- Does not build Android-specific targets (those require Android NDK — Phase 7 release pipeline).
