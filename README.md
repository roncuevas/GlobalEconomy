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

Con el servidor ejecutándose, el smoke test de contrato se puede repetir con `node scripts/api-smoke.mjs`. Para una corrida aislada en otro puerto y directorio, usa `GLOBALECONOMY_PORT=8090 GLOBALECONOMY_DATA_DIR=/tmp/globaleconomy-smoke cargo run --release` y, en otra terminal, `GLOBALECONOMY_BASE_URL=http://127.0.0.1:8090 node scripts/api-smoke.mjs`.

## API mínima

- `POST /api/runs` crea una corrida.
- `GET /api/runs` lista corridas activas con tick, hash de configuración y ticks disponibles para replay.
- `POST /api/runs/compare` compara métricas de varias corridas en un tick común.
- `POST /api/runs/{id}/step` avanza exactamente un tick.
- `POST /api/runs/{id}/start` y `/pause` controlan el estado.
- `POST /api/runs/{id}/policy` aplica una política monetaria en tiempo real.
- `POST /api/runs/{id}/policy/next` aplica el siguiente hito pendiente sin avanzar el tick.
- `POST /api/runs/{id}/policy-mode` cambia entre `manual` y `automatic`.
- `POST /api/runs/{id}/shock` aplica manualmente un shock sintético de oferta, banca, activos o salud en una región.
- `POST /api/runs/{id}/risk/countermeasure` aplica la contramedida del riesgo abierto más reciente.
- `GET /api/runs/{id}/snapshot` devuelve el estado visual reducido.
- `GET /api/runs/{id}/history` devuelve el historial local acotado de snapshots.
- `GET /api/runs/{id}/metrics` devuelve solo la serie compacta de métricas persistidas, sin agentes ni conexiones.
- `GET /api/runs/{id}/metrics.parquet` descarga esa serie en Parquet columnar para DuckDB, Python o notebooks.
- `GET /api/runs/{id}/replay/{tick}` recupera un snapshot histórico exacto; devuelve `410` si ya salió de la ventana local.
- `GET /api/runs/{id}/events` y `/transactions` aceptan filtros por tick, tipo/operación, entidad, región, riesgo y rango monetario.
- `POST /api/runs/{id}/compliance/{alert_id}/resolve` marca una alerta AML/fiscal como revisada y conserva el cambio en el replay.
- `GET /api/runs/{id}/stream` emite por WebSocket mensajes `snapshot`, `metric_update`, `transaction`, `alert` y `run_status`.

El contrato completo está en [`schemas/openapi.yaml`](schemas/openapi.yaml), incluyendo control de corrida, políticas, riesgos, eventos y WebSocket. Para escenarios pequeños existe un adaptador WASM opcional en [`wasm/README.md`](wasm/README.md), que reutiliza el motor Rust sin duplicar reglas.

El endpoint de creación acepta `seed`, `model_version`, `agent_count`, `regions`, `ticks`, `policy_config` y `scenario_config`. El límite defensivo actual es 100,000 agentes; el valor por defecto es 10,000.

Para repetir la medición de escala del motor sin levantar el servidor, ejecuta `cargo run --release --bin scale-benchmark`. El comando mide inicialización y un tick determinista para 1,000 y 10,000 agentes, e imprime también el tamaño del snapshot y las transacciones generadas. La referencia inicial es mantener un tick de 10,000 agentes por debajo de 100 ms en una máquina de desarrollo; los tiempos dependen del hardware.

El servidor mantiene los últimos 240 snapshots por corrida en memoria para replay local y comparación rápida. El estado actual se serializa en `data/runs/run-{id}.json` usando escritura temporal y renombrado atómico; el historial completo se agrega en `run-{id}.snapshots.jsonl`, una línea por snapshot, sin reescribir toda la corrida. Cada snapshot nuevo escribe también su offset en `run-{id}.snapshots.index`, de modo que `/replay/{tick}` puede recuperar un tick antiguo sin deserializar todo el log; los logs anteriores sin índice conservan un fallback compatible por escaneo. Puedes cambiar la carpeta con `GLOBALECONOMY_DATA_DIR`. La serie compacta se puede exportar a Parquet y consultar localmente, por ejemplo: `duckdb -c "SELECT tick, gdp, gini, money_supply FROM read_parquet('run-<id>-metrics.parquet') ORDER BY tick"`.

## Diseño del MVP

El motor está aislado de Axum en `src/lib.rs`. La corrida evolutiva comienza con el mismo patrimonio para todos y genera desigualdad únicamente después de que avanzan los ticks. `monetary_history` registra 69 hitos sintéticos inspirados en la historia monetaria: dinero mercancía y acuñación, banca central y reservas, patrón oro y convertibilidad, guerras y shocks petroleros, Bretton Woods, SDR, tipos flotantes, Plaza/Louvre, ERM y euro, metas de inflación, agregados monetarios, controles y liberalización del crédito, burbujas, crisis asiática y global, QE, LTRO, OMT, tipos cero/negativos, orientación futura, criptoactivos, stablecoins, pagos instantáneos, CBDC, QT y política macroprudencial. En modo manual los nodos continúan evolucionando entre políticas hasta que el usuario aplica el siguiente hito; en modo automático se respeta la agenda de ticks. Los eventos con riesgo sistémico se pueden activar como catástrofes opcionales. El snapshot expone el banco central, los mercados regionales, los eventos de emisión y la riqueza por nodo.

El ciclo institucional inicial también crea empresas, bancos, gobierno, banco central y mercados regionales. Cada tick registra nóminas, impuestos, transferencias y compras como transacciones explicables; los balances de bancos y gobierno se reconcilian después del ciclo. `RunConfig.institutions` permite ajustar el número de empresas, bancos y sectores sin cambiar el contrato existente.

El ciclo de crédito añade préstamos endógenos cuando una persona necesita liquidez o una empresa no cubre su capital de trabajo, registra creación monetaria, amortizaciones y deuda por entidad, y liquida empresas cuando la deuda supera de forma extrema su caja. Las quiebras generan eventos y transacciones de liquidación explicables; los límites de crédito parten de reservas y préstamos de cada banco.

La cronología toma como referencias institucionales la historia de la Reserva Federal, la evolución del Banco de Inglaterra, la línea histórica del FMI sobre Bretton Woods y los SDR, y la creación del euro documentada por el BCE. Es una traducción didáctica al laboratorio, no una reconstrucción cuantitativa de una economía histórica real.

`catastrophes` es opcional y reproducible por semilla. Cuando está activo, se aplican además crisis bancaria, burbuja inmobiliaria, crisis de deuda soberana, shock sanitario e inflación de oferta; cuando está apagado, esos eventos permanecen visibles como escenarios alternativos pero no alteran la corrida.

Antes de cada política pendiente el motor puede detectar un precursor adverso: falsificación y pérdida de confianza, corrida bancaria, fuga de reservas, inflación de oferta, burbuja y apalancamiento, ataque cambiario, estrés soberano, congelamiento de liquidez o problemas de integridad de pagos. Cada alerta conserva causa, señal de detección, severidad, contramedida y marcos métricos antes/después de la política y de la respuesta. La lógica es sintética y defensiva; no contiene instrucciones operativas para cometer delitos.

El módulo de integridad económica analiza las transacciones del tick con reglas transparentes: crédito desproporcionado respecto al ingreso declarado, salidas incompatibles con patrimonio e ingresos, ventas empresariales sin caja suficiente, estructuración de transferencias cerca de un umbral sintético, ciclos de transferencias y volumen entre regiones sin actividad compatible. Cada `ComplianceAlert` conserva categoría (`aml` o `tax_evasion`), severidad, motivo, entidad y los IDs de transacciones relacionadas. El snapshot expone los casos recientes y las métricas `aml_alerts` y `tax_alerts`; son señales sintéticas para detección defensiva, no perfiles de personas reales.

La red no se renderiza como cientos de elementos SVG/React. React conserva los controles y las métricas; PixiJS v8 dibuja nodos y conexiones en un canvas WebGL, dejando preparado el camino para 10,000+ agentes. PixiJS documenta WebGL/WebGL2 como renderer estable recomendado para producción y WebGPU como opción más nueva. La UI también permite seleccionar corridas y comparar sus métricas en ticks históricos; la consulta recupera snapshots antiguos desde el log JSONL cuando ya no están en la ventana rápida.

La cabecera de la UI permite cambiar la semilla reproducible y elegir 1,000, 10,000 o 100,000 agentes antes de crear una corrida. La vista rápida filtra nodos por tipo, región, sector, patrimonio y riesgo sin mutar el motor. El panel de política permite experimentar también con impuesto sobre ingresos y transferencias por persona en tiempo real.

La configuración `scenario_config.evolution` activa estrategias explicables (`saver`, `consumer`, `entrepreneur`) que se ajustan según patrimonio, necesidad e ingreso, con una tasa de mutación reproducible. Con `catastrophes` activo, `shock_probability` e `shock_intensity` generan shocks externos reproducibles por semilla —oferta, corrida bancaria, burbuja de activos o crisis sanitaria— y el snapshot conserva región, impacto y causa. La evolución y los shocks se aplican dentro del ciclo determinista, por lo que también forman parte del replay.

La ruta crítica de cumplimiento indexa las transacciones del tick por entidad, evitando revisar toda la actividad para cada persona. En el entorno de desarrollo, un binario `--release` procesó y devolvió un snapshot de 10,000 agentes en 33 ms por `POST /step` (payload de aproximadamente 2.66 MB). En el canvas, las conexiones PixiJS se agrupan por tipo de flujo y la selección usa una rejilla espacial, reduciendo trabajo por frame sin alterar el resultado económico.

