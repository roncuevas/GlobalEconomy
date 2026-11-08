# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Docentes y estudiantes de economía y finanzas. El docente proyecta el laboratorio en el aula o prepara escenarios; el estudiante lo explora en clase o en tareas para observar cómo emergen la desigualdad, el crédito, las crisis y los efectos de la política monetaria.

## Product Purpose

GlobalEconomy es un laboratorio visual de economía basada en agentes. Simula personas, empresas, bancos, gobierno, banco central y mercados regionales que evolucionan tick a tick desde una igualdad inicial de patrimonio. El éxito tiene dos caras con el mismo peso:

- **Comprensión causal:** el usuario puede explicar por qué una política, un shock o el ciclo de crédito produjo un resultado, siguiendo transacciones, eventos y alertas explicables.
- **Experimentación reproducible:** el usuario compara escenarios con rigor gracias a la semilla, el replay de ticks históricos, la comparación entre corridas y la exportación a Parquet.

## Positioning

Es un motor determinista en el que cada efecto es trazable: la misma configuración, semilla y versión producen la misma corrida, y toda nómina, impuesto, crédito o quiebra queda registrada con su motivo. Junto a esto ofrece una cronología didáctica de 69 hitos de la historia monetaria que se aplican manualmente o según una agenda.

## Operating Context

- Se usa en pantallas grandes, como una laptop o el proyector del aula.
- Flujo típico: crear una corrida (semilla, número de agentes, regiones) → avanzar o reproducir ticks → aplicar políticas, shocks o contramedidas → inspeccionar la red, las métricas y las alertas → comparar corridas o exportar métricas a Parquet para DuckDB o notebooks.
- El API local (Axum, `:8080`) y la UI (Vite + React + PixiJS, `:5173`) corren en la máquina del usuario. Para escenarios pequeños existe un adaptador WASM opcional.

## Capabilities and Constraints

- **Solo español:** la interfaz y los textos permanecen en español; no se requiere i18n.
- **Escritorio primero:** el diseño se optimiza para pantallas de escritorio y proyector; en móvil basta con que no se rompa.
- **Escala de 100,000 agentes:** la UI debe seguir fluida con redes de hasta 100k agentes (10k por defecto). La red se dibuja en un canvas WebGL (PixiJS v8), nunca como elementos DOM o SVG por nodo.
- Determinismo y replay son parte del contrato: ningún trabajo de UI puede alterar el resultado económico de una corrida.
- Terminología establecida en la UI: tick, corrida, semilla, u. m. (unidades monetarias sintéticas), PIB, Gini, AML, evasión fiscal, contramedida, modo Manual/Automática, catástrofes opcionales.
- Todos los datos son sintéticos. La cronología monetaria es una traducción didáctica, no una reconstrucción cuantitativa de economías reales, y las alertas AML o fiscales son señales defensivas, no perfiles de personas reales.

## Evidence on Hand

- El contrato del API está en `schemas/openapi.yaml`, y el smoke test en `scripts/api-smoke.mjs`.
- La referencia de rendimiento es el benchmark de escala (`cargo run --release --bin scale-benchmark`). Según el README, un tick de 10k agentes tarda unos 33 ms en `--release`.
- No hay testimonios, usuarios, instituciones ni casos de uso documentados; ningún trabajo futuro debe inventarlos.

## Product Principles

1. **Todo efecto tiene una causa visible.** Cualquier cambio en métricas o en la red debe poder rastrearse hasta transacciones, eventos o políticas concretas.
2. **Reproducible antes que espectacular.** La semilla, el replay y la comparación son la base del experimento; la UI los hace evidentes sin sacrificarlos.
3. **Didáctico, nunca predictivo.** Explicar términos y mecanismos para estudiantes, sin presentar la simulación como pronóstico ni como datos reales.
4. **Escala sin fricción.** Lo que funciona con 1,000 agentes debe seguir legible y fluido con 100,000.
