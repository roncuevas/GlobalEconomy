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

El motor está aislado de Axum en `src/lib.rs`. La corrida evolutiva comienza con el mismo patrimonio para todos y genera desigualdad únicamente después de que avanzan los ticks. `monetary_history` registra más de treinta hitos sintéticos inspirados en la historia monetaria: unidad de cuenta, moneda metálica, papel convertible, patrón oro, financiación de guerra, crisis bancaria, Bretton Woods, fin de la convertibilidad, dinero fiduciario, crédito bancario, deuda soberana, QE, reducción de balance y dinero digital. El snapshot expone el banco central, los mercados regionales, los eventos de emisión y la riqueza por nodo.

`catastrophes` es opcional y reproducible por semilla. Cuando está activo, se aplican además crisis bancaria, burbuja inmobiliaria, crisis de deuda soberana, shock sanitario e inflación de oferta; cuando está apagado, esos eventos permanecen visibles como escenarios alternativos pero no alteran la corrida.

La red no se renderiza como cientos de elementos SVG/React. React conserva los controles y las métricas; PixiJS v8 dibuja nodos y conexiones en un canvas WebGL, dejando preparado el camino para 10,000+ agentes. PixiJS documenta WebGL/WebGL2 como renderer estable recomendado para producción y WebGPU como opción más nueva. Empresas, bancos comerciales, quiebras, AML/fiscal avanzado, Parquet/DuckDB y WASM quedan como extensiones posteriores del plan.

