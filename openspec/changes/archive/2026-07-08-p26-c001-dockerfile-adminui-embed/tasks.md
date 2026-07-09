# Tasks — p26-c001-dockerfile-adminui-embed

- [x] 1. Add the Node 24 build stage + `COPY --from` of `admin-ui/dist` into the Rust builder before `cargo build`; copy root pnpm manifests + admin-ui + workspace member sources; `pnpm install --frozen-lockfile` + `pnpm --dir admin-ui build` (frf-wasm stub path).
- [x] 2. Verify `docker compose -f compose.yml -f compose.sovereign.yml build gateway` succeeds (the image compiles with the admin UI embedded). QA gate.
