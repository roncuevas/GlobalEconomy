# GlobalEconomy WASM

Adaptador opcional para ejecutar escenarios pequeños directamente en el navegador.
Usa el mismo `Simulation` determinista del motor Rust; no contiene una segunda implementación de las reglas.

## Compilar

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo build --manifest-path wasm/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web/src/wasm \
  target/wasm32-unknown-unknown/release/globaleconomy_wasm.wasm
```

La API exporta `new(config_json)`, `step_json()`, `snapshot_json()` y `tick()`. El adaptador está pensado para corridas pequeñas; las corridas grandes continúan usando el servidor nativo y WebSocket.
