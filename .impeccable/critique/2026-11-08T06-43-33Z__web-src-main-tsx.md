---
target: web/src/main.tsx
total_score: 18
max_score: 40
na_heuristics: 
p0_count: 1
p1_count: 3
target_identity: "file:/Users/roncuevas/Documents/GitHub/GlobalEconomy/web/src/main.tsx"
target_fingerprint: "sha256:26d691cfc75509c5b29f0d95cedf79a8c25e4a25b083c7d9e33bfa4d1e88e09f"
target_path: /Users/roncuevas/Documents/GitHub/GlobalEconomy/web/src/main.tsx
timestamp: 2026-11-08T06-43-33Z
slug: web-src-main-tsx
---
# Crítica: web/src/main.tsx (Operate) — 18/40, Deficiente
Method: dual-agent. Nota: binario release previo a feat(metrics) → NaN% en 3 KPIs (artefacto de build; la UI no lo protege).

## Heurísticas
1 Estado 2 · 2 Lenguaje 2 · 3 Control 2 · 4 Consistencia 2 · 5 Prevención 1 · 6 Reconocimiento 3 · 7 Eficiencia 1 · 8 Minimalismo 2 · 9 Errores 0 · 10 Ayuda 3

## Especificidad
Piel visual propia (serif, papel/salvia, línea de tiempo, glosario); estructura genérica de dashboard; causa y efecto dispersos ~2000px. Detector: estático 2 (style.css overused-font Inter, side-tab .phase); URL 41 (20 low-contrast, 16 tiny-text, hero-eyebrow-chip, cream-palette, straggler); con corrida 178 (hitos opacity .42). FP: content-hidden-at-rest (tooltips), side-tab, network-hint.

## Problemas prioritarios
1. [P0] Sliders hacen POST de política en cada paso de arrastre (main.tsx:192, :225) → contamina replay. Borrador local + confirmar. /impeccable harden
2. [P1] Proyector 1280x720: mapa/política/alerta bajo el pliegue; texto 10–13px; grises 3.3–3.95:1. Cabecera compacta, riel "ahora", 4 KPIs, ≥16px. /impeccable layout, typeset
3. [P1] IA: acciones que mutan la corrida dentro de "Vista rápida"; shocks al fondo; transacciones sin filtrar por nodo. Zonas Experimento/Intervenciones/Observación. /impeccable distill, layout
4. [P1] Sin estados de carga/error/vacío; NaN% renderizado. Formateador único, errores en línea, estado vacío. /impeccable harden, onboard
5. [P2] ~148 Tab stops de tooltips, nombres duplicados, canvas sin alternativa, botones "+ − ⛶". /impeccable audit

## Personas
Alex: sin atajos, sin escenarios nombrados, semilla oculta, aristas recortadas a 2500. Sam: Tab stops, mapa solo puntero, color como único canal. Profesora: mapa bajo el pliegue, ayuda solo hover, sin carga, Nueva corrida sin confirmar, pantalla completa pierde contexto.

## Menores
Líneas sobre valores en gráficas; Gini plano por escala desde 0; caja doble en compliance-summary; timeline sin "ir a fase actual"; Intensidad sin Tip; "General" vs "Sector N"; favicon 404; Play/Step en inglés.

## Preguntas
Efecto de política en el mapa; pantalla de "clase" por defecto; posición que codifique riqueza/región.
