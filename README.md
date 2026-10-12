# GlobalEconomy

Laboratorio visual de economía basada en agentes. El MVP usa datos sintéticos y un motor determinista: la misma configuración, semilla y versión producen la misma corrida.

## Arranque local

```bash
cargo test
cargo run
# en otra terminal
cd web && npm install && npm run dev
```

El API queda en `http://localhost:8080`; la UI en `http://localhost:5173`.

## API mínima

- `POST /api/runs` crea una corrida.
- `POST /api/runs/{id}/step` avanza exactamente un tick.
- `POST /api/runs/{id}/start` y `/pause` controlan el estado.
- `POST /api/runs/{id}/policy` aplica una política monetaria en tiempo real.
- `GET /api/runs/{id}/snapshot` devuelve el estado visual reducido.
- `GET /api/runs/{id}/events` devuelve eventos del último tick.
- `GET /api/runs/{id}/stream` emite snapshots por WebSocket.

El endpoint de creación acepta `seed`, `model_version`, `agent_count`, `regions`, `ticks`, `policy_config` y `scenario_config`. El límite defensivo actual es 100,000 agentes; el valor por defecto es 10,000.

## Diseño del MVP

El motor está aislado de Axum en `src/lib.rs`. La primera versión implementa personas, empleo, salarios, necesidades, compras, impuestos, transferencias, métricas de riqueza/Gini, oferta monetaria, tasa de interés, inyección de liquidez y riesgo explicable básico. El snapshot visual incluye nodos de personas, mercados regionales y aristas de comercio/mercado. Empresas, bancos, quiebras, AML/fiscal avanzado, Parquet/DuckDB y WASM quedan como extensiones posteriores del plan.

