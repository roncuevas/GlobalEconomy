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
- `GET /api/runs` lista corridas activas con tick, hash de configuración y ticks disponibles para replay.
- `POST /api/runs/compare` compara métricas de varias corridas en un tick común.
- `POST /api/runs/{id}/step` avanza exactamente un tick.
- `POST /api/runs/{id}/start` y `/pause` controlan el estado.
- `POST /api/runs/{id}/policy` aplica una política monetaria en tiempo real.
- `POST /api/runs/{id}/policy/next` aplica el siguiente hito pendiente sin avanzar el tick.
- `POST /api/runs/{id}/policy-mode` cambia entre `manual` y `automatic`.
- `POST /api/runs/{id}/risk/countermeasure` aplica la contramedida del riesgo abierto más reciente.
- `GET /api/runs/{id}/snapshot` devuelve el estado visual reducido.
- `GET /api/runs/{id}/history` devuelve el historial local acotado de snapshots.
- `GET /api/runs/{id}/replay/{tick}` recupera un snapshot histórico exacto; devuelve `410` si ya salió de la ventana local.
- `GET /api/runs/{id}/events` devuelve eventos del último tick.
- `GET /api/runs/{id}/transactions` devuelve las transferencias institucionales del último tick.
- `GET /api/runs/{id}/stream` emite snapshots por WebSocket.

El endpoint de creación acepta `seed`, `model_version`, `agent_count`, `regions`, `ticks`, `policy_config` y `scenario_config`. El límite defensivo actual es 100,000 agentes; el valor por defecto es 10,000.

El servidor mantiene los últimos 240 snapshots por corrida en memoria para replay local y comparación rápida. El estado actual se serializa en `data/runs/run-{id}.json` usando escritura temporal y renombrado atómico; el historial completo se agrega en `run-{id}.snapshots.jsonl`, una línea por snapshot, sin reescribir toda la corrida. Al reiniciar se restaura la ventana rápida y `/replay/{tick}` puede recuperar ticks antiguos directamente del log. Puedes cambiar la carpeta con `GLOBALECONOMY_DATA_DIR`. SQLite/Parquet queda preparado como evolución para consultas analíticas y compresión por lotes.

## Diseño del MVP

El motor está aislado de Axum en `src/lib.rs`. La corrida evolutiva comienza con el mismo patrimonio para todos y genera desigualdad únicamente después de que avanzan los ticks. `monetary_history` registra 69 hitos sintéticos inspirados en la historia monetaria: dinero mercancía y acuñación, banca central y reservas, patrón oro y convertibilidad, guerras y shocks petroleros, Bretton Woods, SDR, tipos flotantes, Plaza/Louvre, ERM y euro, metas de inflación, agregados monetarios, controles y liberalización del crédito, burbujas, crisis asiática y global, QE, LTRO, OMT, tipos cero/negativos, orientación futura, criptoactivos, stablecoins, pagos instantáneos, CBDC, QT y política macroprudencial. En modo manual los nodos continúan evolucionando entre políticas hasta que el usuario aplica el siguiente hito; en modo automático se respeta la agenda de ticks. Los eventos con riesgo sistémico se pueden activar como catástrofes opcionales. El snapshot expone el banco central, los mercados regionales, los eventos de emisión y la riqueza por nodo.

El ciclo institucional inicial también crea empresas, bancos, gobierno, banco central y mercados regionales. Cada tick registra nóminas, impuestos, transferencias y compras como transacciones explicables; los balances de bancos y gobierno se reconcilian después del ciclo. `RunConfig.institutions` permite ajustar el número de empresas, bancos y sectores sin cambiar el contrato existente.

El ciclo de crédito añade préstamos endógenos cuando una persona necesita liquidez o una empresa no cubre su capital de trabajo, registra creación monetaria, amortizaciones y deuda por entidad, y liquida empresas cuando la deuda supera de forma extrema su caja. Las quiebras generan eventos y transacciones de liquidación explicables; los límites de crédito parten de reservas y préstamos de cada banco.

La cronología toma como referencias institucionales la historia de la Reserva Federal, la evolución del Banco de Inglaterra, la línea histórica del FMI sobre Bretton Woods y los SDR, y la creación del euro documentada por el BCE. Es una traducción didáctica al laboratorio, no una reconstrucción cuantitativa de una economía histórica real.

`catastrophes` es opcional y reproducible por semilla. Cuando está activo, se aplican además crisis bancaria, burbuja inmobiliaria, crisis de deuda soberana, shock sanitario e inflación de oferta; cuando está apagado, esos eventos permanecen visibles como escenarios alternativos pero no alteran la corrida.

Antes de cada política pendiente el motor puede detectar un precursor adverso: falsificación y pérdida de confianza, corrida bancaria, fuga de reservas, inflación de oferta, burbuja y apalancamiento, ataque cambiario, estrés soberano, congelamiento de liquidez o problemas de integridad de pagos. Cada alerta conserva causa, señal de detección, severidad, contramedida y marcos métricos antes/después de la política y de la respuesta. La lógica es sintética y defensiva; no contiene instrucciones operativas para cometer delitos.

El módulo de integridad económica analiza las transacciones del tick con reglas transparentes: crédito desproporcionado respecto al ingreso declarado, salidas incompatibles con patrimonio e ingresos y ventas empresariales sin caja suficiente. Cada `ComplianceAlert` conserva categoría (`aml` o `tax_evasion`), severidad, motivo, entidad y los IDs de transacciones relacionadas. El snapshot expone los casos recientes y las métricas `aml_alerts` y `tax_alerts`; son señales sintéticas para detección defensiva, no perfiles de personas reales.

La red no se renderiza como cientos de elementos SVG/React. React conserva los controles y las métricas; PixiJS v8 dibuja nodos y conexiones en un canvas WebGL, dejando preparado el camino para 10,000+ agentes. PixiJS documenta WebGL/WebGL2 como renderer estable recomendado para producción y WebGPU como opción más nueva. Empresas, bancos comerciales, quiebras, AML/fiscal avanzado, Parquet/DuckDB y WASM quedan como extensiones posteriores del plan.

