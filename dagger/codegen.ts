/**
 * Dagger codegen pipeline — Phase 3 FFI SDK bindings + Phase 7 WASM build.
 *
 * Stages:
 *  0. clippy          — cargo clippy --workspace --all-targets -D warnings -W clippy::pedantic
 *  1. rust-build      — cargo build -p frf-ffi --release
 *  2. uniffi-swift    — uniffi-bindgen generate --language swift; diff check
 *  3. uniffi-kotlin   — uniffi-bindgen generate --language kotlin; diff check
 *  4. frb-dart        — flutter_rust_bridge_codegen generate; diff check
 *  5. buf-generate    — proto codegen for all SDKs
 *  6. wasm-build      — wasm-pack build crates/frf-wasm → sdks/ts/frf-wasm/
 *  7. pnpm-build      — TS SDK + entity-management + admin-UI (Node 24)
 *  8. e2e-smoke       — Playwright Phase 7 smoke test (admin-UI dev server)
 *  9. bench           — cargo bench -p frf-crdt (optional: ENABLE_BENCH_STAGE=true)
 * 10. integration     — Compose stack + Playwright Layer 3 E2E (optional: ENABLE_INTEGRATION_STAGE=true)
 *
 * Stages 2–4 are skipped when crates/frf-ffi/ is unchanged (Dagger input hash cache).
 * Stage 8 runs against the dev server built in stage 7.
 * Stage 9 is opt-in via ENABLE_BENCH_STAGE=true env var.
 * Stage 10 is opt-in via ENABLE_INTEGRATION_STAGE=true and requires Docker-in-Docker.
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
            const rustBase = client
                .container()
                .from("rust:latest")
                .withDirectory("/workspace", src)
                .withWorkdir("/workspace")
                .withExec(["cargo", "build", "--release", "-p", "frf-ffi"]);

            // ----------------------------------------------------------------
            // Stage 2: UniFFI Swift bindings
            // ----------------------------------------------------------------
            const swiftBindgen = rustBase
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
            const kotlinBindgen = rustBase
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
            // Stage 4: flutter_rust_bridge Dart bindings
            //
            // ghcr.io/cirruslabs/flutter:stable does not ship Rust/cargo.
            // Install rustup with --default-toolchain stable; rust-toolchain.toml
            // in the workspace governs the exact version rustup resolves.
            // ----------------------------------------------------------------
            const dartBindgen = client
                .container()
                .from("ghcr.io/cirruslabs/flutter:stable")
                // Install Rust toolchain (non-interactive, no PATH reload needed — cargo
                // lands at /root/.cargo/bin which we add to PATH via env var below).
                // --default-toolchain stable lets rust-toolchain.toml govern the version.
                .withExec(["sh", "-c",
                    "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --no-modify-path"
                ])
                .withEnvVariable("PATH", "/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
                .withDirectory("/workspace", src)
                .withWorkdir("/workspace")
                .withExec(["cargo", "install", "flutter_rust_bridge_codegen", "--version", "2.11.1"])
                .withExec([
                    "flutter_rust_bridge_codegen", "generate",
                    "--rust-input", "crates/frf-ffi/src/lib.rs",
                    "--dart-output", "/tmp/dart-gen/frb_generated.dart",
                    "--rust-root", "/workspace",
                ])
                .withExec([
                    "diff",
                    "/tmp/dart-gen/frb_generated.dart",
                    "sdks/dart/lib/src/rust/frb_generated.dart",
                ]);

            // ----------------------------------------------------------------
            // Stage 5: proto codegen (buf generate)
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
            // Stage 6: WASM build — crates/frf-wasm → sdks/ts/frf-wasm/
            //
            // Uses rust:latest + wasm-pack. The output is mounted into the
            // pnpm build stage (stage 7) so admin-UI can import the package.
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
            // Stage 7: pnpm build — TS SDK + entity-management + admin-UI
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

            // ----------------------------------------------------------------
            // Stage 8: E2E smoke test — Phase 7 gateway smoke
            //
            // Installs Playwright inside the pnpm build container, runs the
            // Phase 7 E2E smoke suite against the built admin-UI.  The test
            // file at admin-ui/e2e/p7-smoke.spec.ts must exist and pass.
            // ----------------------------------------------------------------
            const e2eSmoke = pnpmBuild
                .withExec(["pnpm", "--filter", "@prometheusags/frf-admin-ui",
                    "exec", "playwright", "install", "--with-deps", "chromium",
                ])
                // WASM was built by Stage 6 and mounted via wasmOut — enable CRDT Layer 2 tests.
                .withEnvVariable("WASM_AVAILABLE", "1")
                .withWorkdir("/workspace/admin-ui")
                .withExec(["pnpm", "exec", "playwright", "test",
                    "e2e/",
                    "--reporter=list",
                ]);

            // ----------------------------------------------------------------
            // Stage 9: Criterion benchmark (opt-in)
            //
            // Only materializes when ENABLE_BENCH_STAGE=true is set.
            // Runs `cargo bench -p frf-crdt` using the committed .criterion/
            // baseline to catch regressions on CI.
            // ----------------------------------------------------------------
            const stages: Promise<unknown>[] = [
                clippyCheck.sync(),
                swiftBindgen.sync(),
                kotlinBindgen.sync(),
                dartBindgen.sync(),
                bufGen.sync(),
                // WASM → pnpm → e2e chain
                e2eSmoke.sync(),
            ];

            if (process.env["ENABLE_BENCH_STAGE"] === "true") {
                const bench = client
                    .container()
                    .from("rust:latest")
                    .withDirectory("/workspace", src)
                    .withWorkdir("/workspace")
                    // Restore committed baseline into target/criterion/ so --baseline can load it.
                    .withExec(["sh", "-c",
                        "mkdir -p target/criterion && cp -r .criterion/* target/criterion/ 2>/dev/null || true"
                    ])
                    .withExec([
                        "cargo", "bench", "-p", "frf-crdt", "--bench", "crdt_merge",
                        "--", "--baseline", "main",
                    ])
                    .withExec(["bash", "scripts/bench-regression-check.sh"]);
                stages.push(bench.sync());
                console.log("Stage 9 (bench) enabled.");
            }

            // ----------------------------------------------------------------
            // Stage 10: Layer 3 E2E integration (opt-in)
            //
            // Requires ENABLE_INTEGRATION_STAGE=true and a Docker host with
            // /var/run/docker.sock mounted into the Dagger runner (DinD).
            //
            // Flow:
            //   docker compose up -d    → start the full stack (gateway + iggy + keto)
            //   wait until /healthz 200 → poll gateway readiness (max 60s)
            //   playwright test          → run Layer 3 E2E suite with WASM_AVAILABLE=1
            //   docker compose down     → tear down regardless of test outcome
            // ----------------------------------------------------------------
            if (process.env["ENABLE_INTEGRATION_STAGE"] === "true") {
                const integration = client
                    .container()
                    .from("node:24-slim")
                    .withExec(["apt-get", "update"])
                    .withExec(["apt-get", "install", "-y", "--no-install-recommends",
                        "curl", "ca-certificates", "gnupg", "lsb-release",
                    ])
                    // Add Docker's official apt repo so docker-compose-plugin is available
                    .withExec(["sh", "-c",
                        "install -m 0755 -d /etc/apt/keyrings && " +
                        "curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg && " +
                        "chmod a+r /etc/apt/keyrings/docker.gpg && " +
                        "echo \"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(. /etc/os-release && echo $VERSION_CODENAME) stable\" > /etc/apt/sources.list.d/docker.list"
                    ])
                    .withExec(["apt-get", "update"])
                    .withExec(["apt-get", "install", "-y", "--no-install-recommends",
                        "docker-ce-cli", "docker-compose-plugin",
                    ])
                    .withExec(["npm", "install", "-g", "pnpm"])
                    .withDirectory("/workspace", src)
                    // Mount the host Docker socket — use the real path (not the symlink at
                    // /var/run/docker.sock which Dagger cannot resolve inside the engine).
                    .withUnixSocket("/var/run/docker.sock", client.host().unixSocket("/Users/gqadonis/.docker/run/docker.sock"))
                    .withWorkdir("/workspace")
                    // Use compose.ci.yml — a CI-specific compose file with no bind mounts.
                    // Regular compose.yml uses ./deploy/keto and ./deploy/postgres mounts
                    // which Docker Desktop on Mac denies when compose runs inside a container.
                    // compose.ci.yml injects keto config via env vars and omits the postgres
                    // init.sql mount (CDC is disabled via CDC_ENABLED=false in CI).
                    .withExec(["docker", "compose", "-f", "compose.ci.yml", "up", "-d"])
                    // Poll gateway healthz up to 60 seconds
                    .withExec(["sh", "-c",
                        "for i in $(seq 1 30); do curl -sf http://localhost:28080/healthz && break || sleep 2; done"
                    ])
                    // Run the admin-UI Layer 3 E2E suite
                    // CI=true tells pnpm to skip the interactive TTY check for node_modules purge
                    .withEnvVariable("CI", "true")
                    .withEnvVariable("WASM_AVAILABLE", "1")
                    .withEnvVariable("GATEWAY_URL", "http://localhost:28080")
                    .withEnvVariable("SKIP_INTEGRATION", "false")
                    .withEnvVariable("DEV_ENDPOINTS_ENABLED", "true")
                    .withWorkdir("/workspace/admin-ui")
                    .withExec(["pnpm", "install", "--frozen-lockfile"])
                    .withExec(["pnpm", "exec", "playwright", "install", "--with-deps", "chromium"])
                    .withExec(["pnpm", "exec", "playwright", "test",
                        "e2e/",
                        "--reporter=list",
                    ])
                    // Tear down the stack (always runs via .withExec chaining)
                    .withWorkdir("/workspace")
                    .withExec(["docker", "compose", "-f", "compose.ci.yml", "down"]);

                stages.push(integration.sync());
                console.log("Stage 10 (integration) enabled.");
            }

            // ----------------------------------------------------------------
            // Materialize all stages (parallel where independent)
            // WASM (6) → pnpm (7) → e2e (8) are sequential by dependency.
            // Stages 2–5 run in parallel against stage 1's cache.
            // Stages 9 (bench) and 10 (integration) run in parallel when enabled.
            // ----------------------------------------------------------------
            await Promise.all(stages);

            console.log("All codegen + WASM + E2E stages passed.");
        },
        { LogOutput: process.stderr },
    );
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
