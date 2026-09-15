/**
 * Dagger build pipeline — FFI SDK bindings, WASM, and package builds.
 *
 * Stages:
 *  0. clippy          — cargo clippy --workspace --all-targets -D warnings -W clippy::pedantic
 *  1. rust-build      — cargo build -p frf-ffi --release
 *  2. uniffi-swift    — uniffi-bindgen generate --language swift; diff check
 *  3. uniffi-kotlin   — uniffi-bindgen generate --language kotlin; diff check
 *  4. buf-generate    — proto codegen for all SDKs
 *  5. wasm-build      — wasm-pack build crates/frf-wasm → sdks/ts/frf-wasm/
 *  6. pnpm-build      — TS SDK + entity-management + admin-UI (Node 24)
 *
 * Runtime tests and benchmarks run only on a local developer/operator host.
 * See dagger/README.md for the direct local entry points. Dart generation also
 * remains local because uniffi-bindgen-dart 0.1.3 output requires documented
 * post-generation patches (ADR-003 and sdks/dart/GENERATED.md).
 *
 * Run:
 *   dagger run ts-node dagger/codegen.ts
 */

import { connect, Client, Container, Directory } from "@dagger.io/dagger";

async function main() {
    await connect(
        async (client: Client) => {
            const src = client.host().directory(".", {
                exclude: [
                    "target/**",
                    "admin-ui/node_modules/**",
                    "sdks/ts/node_modules/**",
                    "sdks/entity-management/node_modules/**",
                    ".git/**",
                ],
            });

            // ----------------------------------------------------------------
            // Stage 0: Clippy workspace lint gate (pedantic, deny warnings)
            //
            // rust:latest always tracks the current stable release.
            // rust-toolchain.toml (channel = "stable", components = [..., "clippy"])
            // is picked up by rustup automatically — no explicit component add needed.
            // ----------------------------------------------------------------
            // Restriction gate (unwrap_used / expect_used) runs on production
            // code only (--lib --bins); test/#[cfg(test)] code may use unwrap()
            // (clippy.toml allow-in-tests). Test targets get a pedantic-only pass.
            const clippyCheck = client
                .container()
                .from("rust:latest")
                .withDirectory("/workspace", src)
                .withWorkdir("/workspace")
                .withExec([
                    "cargo", "clippy", "--workspace", "--lib", "--bins",
                    "--", "-D", "warnings", "-W", "clippy::pedantic",
                ])
                .withExec([
                    "cargo", "clippy", "--workspace", "--tests",
                    "--", "-D", "warnings", "-W", "clippy::pedantic",
                ]);

            // ----------------------------------------------------------------
            // Stage 1: build frf-ffi release dylib
            // ----------------------------------------------------------------
            const rustBuild = client
                .container()
                .from("rust:latest")
                .withDirectory("/workspace", src)
                .withWorkdir("/workspace")
                .withExec(["cargo", "build", "--release", "-p", "frf-ffi"]);

            // ----------------------------------------------------------------
            // Stage 2: UniFFI Swift bindings
            // ----------------------------------------------------------------
            const swiftBindgen = rustBuild
                .withExec([
                    "cargo", "run", "--bin", "uniffi-bindgen", "--",
                    "generate",
                    "--library", "target/release/libfrf_ffi.so",
                    "--language", "swift",
                    "--out-dir", "/tmp/swift-gen",
                ])
                .withExec([
                    "diff",
                    "/tmp/swift-gen/frf.swift",
                    "sdks/swift/Sources/FrfClient/frf.swift",
                ]);

            // ----------------------------------------------------------------
            // Stage 3: UniFFI Kotlin bindings
            // ----------------------------------------------------------------
            const kotlinBindgen = rustBuild
                .withExec([
                    "cargo", "run", "--bin", "uniffi-bindgen", "--",
                    "generate",
                    "--library", "target/release/libfrf_ffi.so",
                    "--language", "kotlin",
                    "--out-dir", "/tmp/kotlin-gen",
                ])
                .withExec([
                    "diff",
                    "/tmp/kotlin-gen/uniffi/frf/frf.kt",
                    "sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt",
                ]);

            // ----------------------------------------------------------------
            // Stage 4: proto codegen (buf generate)
            //
            // buf.gen.yaml lives in proto/ and references local Go plugins
            // (protoc-gen-go, protoc-gen-connect-go). We use a golang base image
            // so those plugins can be installed via `go install`, then overlay
            // the buf CLI binary from bufbuild/buf:latest.
            // ----------------------------------------------------------------
            const bufBase = client
                .container()
                .from("bufbuild/buf:latest");

            const bufGen = client
                .container()
                .from("golang:1.24-bookworm")
                // Copy buf binary from the official buf image
                .withFile("/usr/local/bin/buf", bufBase.file("/usr/local/bin/buf"))
                // Install local protoc plugins needed by buf.gen.yaml
                .withExec(["go", "install", "google.golang.org/protobuf/cmd/protoc-gen-go@latest"])
                // v1.17.0 is the last release supporting Go 1.24
                .withExec(["go", "install", "connectrpc.com/connect/cmd/protoc-gen-connect-go@v1.17.0"])
                .withDirectory("/workspace", src)
                // buf generate must run from the directory that contains buf.gen.yaml
                .withWorkdir("/workspace/proto")
                .withExec(["buf", "generate"]);

            // ----------------------------------------------------------------
            // Stage 5: WASM build — crates/frf-wasm → sdks/ts/frf-wasm/
            //
            // Uses rust:latest + wasm-pack. The output is mounted into the
            // pnpm build stage (stage 6) so admin-UI can import the package.
            // ----------------------------------------------------------------
            const wasmBuild: Container = client
                .container()
                .from("rust:latest")
                .withExec(["apt-get", "update"])
                .withExec(["apt-get", "install", "-y", "--no-install-recommends",
                    "curl", "ca-certificates", "pkg-config", "build-essential", "wget",
                ])
                // Install binaryen 116 — supports the bulk-memory proposal used by Loro CRDT.
                // The wasm-pack-bundled wasm-opt (v105) predates bulk-memory and fails
                // to validate WASM output that uses it; binaryen 116 shadows it via PATH.
                .withExec(["sh", "-c",
                    "BINARYEN_VER=version_116 && " +
                    "wget -q https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VER}/binaryen-${BINARYEN_VER}-x86_64-linux.tar.gz -O /tmp/binaryen.tar.gz && " +
                    "tar -xzf /tmp/binaryen.tar.gz -C /usr/local --strip-components=1 && " +
                    "wasm-opt --version"
                ])
                // Install wasm-pack
                .withExec(["sh", "-c",
                    "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
                ])
                // Install wasm32 target
                .withExec(["rustup", "target", "add", "wasm32-unknown-unknown"])
                .withDirectory("/workspace", src)
                .withWorkdir("/workspace/crates/frf-wasm")
                .withExec([
                    "wasm-pack", "build",
                    "--target", "web",
                    "--out-dir", "/workspace/sdks/ts/frf-wasm",
                    "--out-name", "frf_wasm",
                    "--release",
                ])
                // Verify wasm-pack produced the expected output artefacts.
                .withExec(["sh", "-c",
                    "test -f /workspace/sdks/ts/frf-wasm/frf_wasm.js && " +
                    "test -f /workspace/sdks/ts/frf-wasm/frf_wasm_bg.wasm || " +
                    "{ echo 'WASM build output missing: expected frf_wasm.js and frf_wasm_bg.wasm'; exit 1; }"
                ])
                // Verify package.json has the correct name field.
                .withExec(["sh", "-c",
                    "command -v jq >/dev/null 2>&1 || apt-get install -y --no-install-recommends jq; " +
                    "jq -e '.name == \"frf-wasm\"' /workspace/sdks/ts/frf-wasm/package.json || " +
                    "{ echo 'frf-wasm package.json name mismatch'; exit 1; }"
                ])
                // Measure WASM binary size; compare against committed baseline if it exists.
                // 150% threshold catches accidental regressions (debug symbols, forgotten strip).
                // To update the baseline: measure locally and commit .wasm-size-baseline.
                .withExec(["sh", "-c",
                    "SIZE=$(wc -c < /workspace/sdks/ts/frf-wasm/frf_wasm_bg.wasm) && " +
                    "echo \"WASM binary size: ${SIZE} bytes\" && " +
                    "if [ -f /workspace/.wasm-size-baseline ]; then " +
                    "  BASELINE=$(cat /workspace/.wasm-size-baseline | tr -d '[:space:]'); " +
                    "  LIMIT=$((BASELINE * 3 / 2)); " +
                    "  if [ \"$SIZE\" -gt \"$LIMIT\" ]; then " +
                    "    echo \"FAIL: WASM size ${SIZE} > 150% of baseline ${BASELINE} (limit: ${LIMIT} bytes)\"; " +
                    "    exit 1; " +
                    "  else " +
                    "    echo \"OK: WASM size ${SIZE} within 150% of baseline ${BASELINE} (limit: ${LIMIT} bytes)\"; " +
                    "  fi; " +
                    "else " +
                    "  echo \"No baseline found — commit .wasm-size-baseline with current SIZE to enable regression guard\"; " +
                    "fi"
                ]);

            // Export built WASM output directory so stage 7 can consume it.
            const wasmOut: Directory = wasmBuild.directory("/workspace/sdks/ts/frf-wasm");

            // ----------------------------------------------------------------
            // Stage 6: pnpm build — TS SDK + entity-management + admin-UI
            //          Uses Node 24 (matches engines.node in package.json).
            //          Mounts WASM output before building admin-UI.
            // ----------------------------------------------------------------
            const pnpmBuild = client
                .container()
                .from("node:24-slim")
                .withExec(["npm", "install", "-g", "pnpm"])
                .withDirectory("/workspace", src)
                // Overlay built WASM package into the workspace
                .withDirectory("/workspace/sdks/ts/frf-wasm", wasmOut)
                .withWorkdir("/workspace")
                .withExec(["pnpm", "install", "--frozen-lockfile"])
                // Lint admin-UI (catches !!process.env coercion and TypeScript issues)
                .withWorkdir("/workspace/admin-ui")
                .withExec(["pnpm", "lint"])
                .withWorkdir("/workspace")
                .withExec(["pnpm", "-r", "build"]);

            const stages: Promise<unknown>[] = [
                clippyCheck.sync(),
                rustBuild.sync(),
                swiftBindgen.sync(),
                kotlinBindgen.sync(),
                bufGen.sync(),
                // WASM output is consumed by the package build.
                pnpmBuild.sync(),
            ];
            await Promise.all(stages);

            console.log("All lint, codegen, WASM, and package build stages passed.");
        },
        { LogOutput: process.stderr },
    );
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
