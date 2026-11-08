# Repository Guidelines

## Project Structure & Module Organization

The deterministic Rust simulation is in `src/lib.rs`; the independent family and inheritance model is in `src/generations.rs`. Axum routes and persistence are in `src/main.rs`, with Generaciones handlers in `src/generations_api.rs`. The React, TypeScript, Zustand, and PixiJS interface is under `web/src/`. HTTP contracts are in `schemas/openapi.yaml`, API smoke checks in `scripts/`, and product context in `README.md` and `PRODUCT.md`. Optional WebAssembly support is in `wasm/`. Put Rust unit tests beside the module; integration tests belong in `tests/`.

## Build, Test, and Development Commands

- `cargo run --release`: start the API at `http://localhost:8080`.
- `cd web && npm install && npm run dev`: install dependencies and run Vite locally.
- `cargo test`: run Rust engine, API, and handler tests.
- `cd web && npx tsc -b`: type-check the frontend; `npm run build` also creates the production bundle.
- `node scripts/api-smoke.mjs`: check the HTTP contract against a running API. Set `GLOBALECONOMY_BASE_URL` to target another port.
- `cargo run --release --bin scale-benchmark`: measure simulation initialization and ticks at increasing population sizes.

## Coding Style & Naming Conventions

Use Rust 2021 conventions: format with `cargo fmt`, use `snake_case` for functions/modules and `UpperCamelCase` for types. Simulation randomness must derive from the configured seed and tick. In the frontend, use `PascalCase` for components and `camelCase` for values and functions. Preserve Spanish UI wording and align API changes with `schemas/openapi.yaml`.

## Testing Guidelines

Add focused Rust unit tests beside the behavior they protect and run `cargo test`. Frontend checks use `npx tsc -b` and the Vite build; there is no frontend test script. For route or schema changes, run the API smoke script against a server. Use a separate port and data directory for isolated checks.

## Commit & Pull Request Guidelines

Use multiple small, atomic commits when work contains independently reviewable changes. Follow Conventional Commits, such as `feat(engine): ...`, `fix(api): ...`, `test(api): ...`, `perf(engine): ...`, and `docs(readme): ...`; keep subjects concise and scoped. Never add the AI agent as a commit co-author. Pull requests should explain user-visible and API behavior changes, list validation commands and outcomes, link relevant issues when available, and include screenshots for UI changes. Update the OpenAPI schema and README when their contracts or documented workflows change.

## Security & Configuration

Keep secrets and local simulation data out of commits. The server stores run state under `data/`; use `GLOBALECONOMY_DATA_DIR` to select a disposable directory for manual or smoke checks. Do not commit generated `target/`, `node_modules/`, or build output.
