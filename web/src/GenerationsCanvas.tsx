import { useEffect, useRef, useState, type ReactNode } from 'react';
import { Application, Container, Graphics, Rectangle, Sprite, Text, type Texture } from 'pixi.js';
import type { Citizen, GenEvent, GenSnapshot } from './generationsTypes';
import './network.css';

export type Layout = 'lineage' | 'wealth';

/** Paleta de linajes contrastada contra el papel (#f1f0eb); se repite a partir de 12 familias. */
export const LINEAGE_COLORS = [0x2e6f95, 0xb5523b, 0x3f7d20, 0x8e5ba8, 0xc98a1b, 0x1f8a87, 0xa23b72, 0x6b4f3a, 0x4a5fc1, 0x7a8b1e, 0xd06a8e, 0x36454f];
export const lineageColor = (lineage: number) => LINEAGE_COLORS[lineage % LINEAGE_COLORS.length];
export const lineageCss = (lineage: number) => `#${lineageColor(lineage).toString(16).padStart(6, '0')}`;

const TEXTURE_RADIUS = 32;
const DEAD_TINT = 0x9aa19c;
const BRASS = 0x9c6838;

type Node = { id: number; sprite: Sprite; fromX: number; fromY: number; fromR: number; toX: number; toY: number; toR: number; x: number; y: number; r: number; born: boolean };
type Flow = { fromX: number; fromY: number; toX: number; toY: number };
type Scene = { app: Application; world: Container; links: Graphics; fx: Graphics; axis: Graphics; labels: Container; sprites: Container; texture: Texture; zoom: number };

const easeInOut = (t: number) => (t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2);
const easeOutBack = (t: number) => 1 + 2.4 * (t - 1) ** 3 + 1.4 * (t - 1) ** 2;
const hash = (id: number) => { let h = Math.imul(id ^ 0x9e3779b9, 0x85ebca6b); h ^= h >>> 13; h = Math.imul(h, 0xc2b2ae35); return ((h ^ (h >>> 16)) >>> 0) / 4294967295; };
const reducedMotion = () => window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;

export function GenerationsCanvas({ snapshot, events, layout, selected, highlightLineage, duration, onSelect, children }: {
  snapshot: GenSnapshot | null; events: GenEvent[]; layout: Layout; selected: number | null; highlightLineage: number | null; duration: number;
  onSelect: (id: number | null) => void; children?: ReactNode;
}) {
  const hostRef = useRef<HTMLDivElement>(null); const sceneRef = useRef<Scene | null>(null); const nodesRef = useRef(new Map<number, Node>());
  const flowsRef = useRef<Flow[]>([]); const startRef = useRef(0); const durationRef = useRef(duration); const maxWealthRef = useRef(100); const runRef = useRef('');
  const propsRef = useRef({ snapshot, events, layout, selected, highlightLineage, onSelect }); const movedRef = useRef(false); const [zoom, setZoom] = useState(1);
  propsRef.current = { snapshot, events, layout, selected, highlightLineage, onSelect }; durationRef.current = duration;

  function setCamera(next: number) { const scene = sceneRef.current; if (!scene) return; scene.zoom = Math.max(.5, Math.min(6, next)); scene.world.scale.set(scene.zoom); setZoom(scene.zoom); }
  function zoomBy(factor: number) { const scene = sceneRef.current; if (scene) setCamera(scene.zoom * factor); }
  function resetCamera() { const scene = sceneRef.current; if (!scene) return; scene.world.position.set(0, 0); setCamera(1); }
  function fullscreen() { hostRef.current?.requestFullscreen?.(); }

  useEffect(() => {
    const host = hostRef.current; if (!host) return; let cancelled = false; let dragging = false; let lastX = 0; let lastY = 0;
    const boot = async () => {
      const app = new Application();
      await app.init({ resizeTo: host, preference: 'webgl', antialias: true, backgroundAlpha: 0, autoDensity: true, resolution: Math.min(window.devicePixelRatio || 1, 2) });
      if (cancelled) { app.destroy(true); return; }
      host.appendChild(app.canvas);
      const dot = new Graphics().circle(TEXTURE_RADIUS, TEXTURE_RADIUS, TEXTURE_RADIUS).fill(0xffffff);
      const texture = app.renderer.generateTexture(dot); dot.destroy();
      const world = new Container(), axis = new Graphics(), labels = new Container(), links = new Graphics(), sprites = new Container(), fx = new Graphics();
      world.addChild(axis, labels, links, sprites, fx); app.stage.addChild(world);
      sceneRef.current = { app, world, links, fx, axis, labels, sprites, texture, zoom: 1 };
      app.renderer.on('resize', () => retarget(true));
      app.ticker.add(frame);
      retarget(true);
    };
    const toWorld = (event: PointerEvent) => { const scene = sceneRef.current!; const rect = host.getBoundingClientRect(); return { x: (event.clientX - rect.left - scene.world.position.x) / scene.zoom, y: (event.clientY - rect.top - scene.world.position.y) / scene.zoom }; };
    const onWheel = (event: WheelEvent) => { event.preventDefault(); zoomBy(event.deltaY < 0 ? 1.14 : .88); };
    const onPointerDown = (event: PointerEvent) => { if ((event.target as HTMLElement).closest('button,a,input,label,.gen-overlay')) return; dragging = true; movedRef.current = false; lastX = event.clientX; lastY = event.clientY; host.setPointerCapture?.(event.pointerId); };
    const onPointerMove = (event: PointerEvent) => { if (!dragging) return; const scene = sceneRef.current; if (!scene) return; const dx = event.clientX - lastX, dy = event.clientY - lastY; if (Math.abs(dx) + Math.abs(dy) > 2) movedRef.current = true; scene.world.position.x += dx; scene.world.position.y += dy; lastX = event.clientX; lastY = event.clientY; };
    const onPointerUp = (event: PointerEvent) => {
      if (!dragging) return; dragging = false; host.releasePointerCapture?.(event.pointerId);
      if (movedRef.current || !sceneRef.current) return;
      const { x, y } = toWorld(event); let best: Node | undefined; let distance = Infinity;
      for (const node of nodesRef.current.values()) { const d = Math.hypot(node.x - x, node.y - y) - node.r; if (d < distance) { distance = d; best = node; } }
      propsRef.current.onSelect(best && distance < 10 / sceneRef.current.zoom ? best.id : null);
    };
    host.addEventListener('wheel', onWheel, { passive: false }); host.addEventListener('pointerdown', onPointerDown); host.addEventListener('pointermove', onPointerMove); host.addEventListener('pointerup', onPointerUp); host.addEventListener('pointercancel', onPointerUp);
    boot();
    return () => { cancelled = true; host.removeEventListener('wheel', onWheel); host.removeEventListener('pointerdown', onPointerDown); host.removeEventListener('pointermove', onPointerMove); host.removeEventListener('pointerup', onPointerUp); host.removeEventListener('pointercancel', onPointerUp); sceneRef.current?.app.destroy(true, { children: true }); sceneRef.current = null; nodesRef.current.clear(); };
  }, []);

  useEffect(() => { retarget(false); }, [snapshot, layout]);

  /** Calcula el destino de cada persona y arranca la transición desde donde se ve ahora. */
  function retarget(jump: boolean) {
    const scene = sceneRef.current; const { snapshot: snap, events: stepEvents, layout: mode } = propsRef.current; if (!scene) return;
    const nodes = nodesRef.current;
    if (!snap) { for (const node of nodes.values()) node.sprite.destroy(); nodes.clear(); flowsRef.current = []; drawAxis(scene, mode, null); return; }
    if (runRef.current !== snap.config_hash) { runRef.current = snap.config_hash; maxWealthRef.current = 100; for (const node of nodes.values()) node.sprite.destroy(); nodes.clear(); resetCamera(); jump = true; }
    const width = scene.app.renderer.width / scene.app.renderer.resolution, height = scene.app.renderer.height / scene.app.renderer.resolution;
    scene.app.stage.hitArea = new Rectangle(0, 0, width, height);
    const living = snap.citizens.filter(c => c.alive);
    maxWealthRef.current = Math.max(maxWealthRef.current, ...living.map(c => c.money));
    const mean = Math.max(snap.metrics.mean_wealth, 1);
    const maxGeneration = Math.max(1, ...snap.citizens.map(c => c.generation));
    const target = (c: Citizen) => position(c, mode, width, height, maxWealthRef.current, maxGeneration);
    const seen = new Set<number>();
    for (const c of snap.citizens) {
      seen.add(c.id); const [toX, toY] = target(c); let node = nodes.get(c.id);
      const toR = c.alive ? Math.max(2.5, Math.min(22, 6 * Math.sqrt(Math.max(c.money, 0) / mean))) : node?.toR ?? 3;
      if (!node) {
        const sprite = new Sprite(scene.texture); sprite.anchor.set(.5); scene.sprites.addChild(sprite);
        const parent = c.parent !== null ? nodes.get(c.parent) : undefined;
        const fromX = jump || !parent ? toX : parent.x, fromY = jump || !parent ? toY : parent.y;
        node = { id: c.id, sprite, fromX, fromY, fromR: jump ? toR : 0, toX, toY, toR, x: fromX, y: fromY, r: jump ? toR : 0, born: !jump };
        nodes.set(c.id, node);
      } else {
        Object.assign(node, { fromX: jump ? toX : node.x, fromY: jump ? toY : node.y, fromR: jump ? toR : node.r, toX, toY, toR, born: false });
      }
    }
    for (const [id, node] of nodes) if (!seen.has(id)) { node.sprite.destroy(); nodes.delete(id); }
    flowsRef.current = [];
    for (const event of stepEvents) {
      if (event.kind !== 'inheritance') continue; const from = nodes.get(event.from); if (!from) continue;
      for (const heir of event.heirs) { const to = nodes.get(heir); if (to) flowsRef.current.push({ fromX: from.fromX, fromY: from.fromY, toX: to.toX, toY: to.toY }); }
    }
    drawAxis(scene, mode, snap);
    startRef.current = jump || reducedMotion() ? -Infinity : performance.now();
  }

  function frame() {
    const scene = sceneRef.current; if (!scene) return;
    const { snapshot: snap, layout: mode, selected: focus, highlightLineage: lineageFocus } = propsRef.current;
    const span = Math.max(durationRef.current, 1); const p = Math.min(1, (performance.now() - startRef.current) / span); const e = easeInOut(p);
    const byId = new Map((snap?.citizens ?? []).map(c => [c.id, c]));
    const family = focus !== null ? familyOf(focus, byId) : null;
    for (const node of nodesRef.current.values()) {
      const c = byId.get(node.id); if (!c) continue;
      node.x = node.fromX + (node.toX - node.fromX) * e; node.y = node.fromY + (node.toY - node.fromY) * e;
      node.r = node.born ? node.toR * Math.max(0, easeOutBack(p)) : node.fromR + (node.toR - node.fromR) * e;
      const sprite = node.sprite; sprite.position.set(node.x, node.y); sprite.scale.set(Math.max(node.r, .01) / TEXTURE_RADIUS);
      const fade = c.alive ? 1 : Math.max(.12, .45 - ((snap!.tick - (c.died_tick ?? snap!.tick)) * .12 + e * .12));
      const dimmed = (family && !family.has(c.id)) || (lineageFocus !== null && c.lineage !== lineageFocus);
      sprite.tint = c.alive ? lineageColor(c.lineage) : DEAD_TINT; sprite.alpha = (c.alive ? .9 : fade) * (dimmed ? .18 : 1);
    }
    const links = scene.links; links.clear();
    for (const node of nodesRef.current.values()) {
      const c = byId.get(node.id); if (!c || c.parent === null) continue; const parent = nodesRef.current.get(c.parent); if (!parent) continue;
      const inFamily = family?.has(c.id) && family.has(c.parent);
      if (mode === 'wealth' && !inFamily) continue;
      const dimmed = (family && !inFamily) || (lineageFocus !== null && c.lineage !== lineageFocus);
      links.moveTo(parent.x, parent.y).lineTo(node.x, node.y).stroke({ color: inFamily ? 0x18201d : lineageColor(c.lineage), width: inFamily ? 1.4 : .8, alpha: dimmed ? .05 : inFamily ? .7 : c.alive ? .32 : .12 });
    }
    const fx = scene.fx; fx.clear();
    for (const node of nodesRef.current.values()) if (node.born && p < 1) fx.circle(node.x, node.y, node.toR + 3 + p * 10).stroke({ color: lineageColor(byId.get(node.id)?.lineage ?? 0), width: 1.5, alpha: (1 - p) * .8 });
    const flowP = Math.min(1, p * 1.15);
    if (flowP < 1) for (const flow of flowsRef.current) for (let k = 0; k < 3; k++) {
      const t = Math.max(0, Math.min(1, flowP * 1.3 - k * .15)); const x = flow.fromX + (flow.toX - flow.fromX) * t, y = flow.fromY + (flow.toY - flow.fromY) * t;
      fx.circle(x, y, 2.6 - k * .5).fill({ color: BRASS, alpha: .95 - k * .25 });
    }
    if (focus !== null) { const node = nodesRef.current.get(focus); if (node) fx.circle(node.x, node.y, node.r + 4).stroke({ color: 0x18201d, width: 2, alpha: .9 }); }
  }

  const population = snapshot?.metrics.population ?? 0;
  const summary = snapshot ? `Mapa de ${layout === 'lineage' ? 'familias y linajes' : 'riqueza por generación'}: ${population} personas vivas en el año ${snapshot.tick}, ${snapshot.metrics.lineages_alive} familias con descendencia viva. El panel «Qué pasó» describe cada cambio en texto.` : 'Mapa vacío hasta crear una sociedad.';
  return <div className="network-canvas gen-canvas" ref={hostRef}>
    <div className="network-surface" role="img" aria-label={summary} />
    {children}
    <div className="network-toolbar" role="toolbar" aria-label="Controles del mapa">
      <button type="button" onClick={() => zoomBy(1.25)} aria-label="Acercar" title="Acercar"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 3v10M3 8h10" stroke="currentColor" strokeWidth="1.6" fill="none" /></svg></button>
      <button type="button" onClick={() => zoomBy(.8)} aria-label="Alejar" title="Alejar"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8h10" stroke="currentColor" strokeWidth="1.6" fill="none" /></svg></button>
      <button type="button" onClick={resetCamera} aria-label="Restablecer vista" title="Restablecer vista"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 8a4.5 4.5 0 1 0 1.4-3.3M3.5 2.5v2.6h2.6" stroke="currentColor" strokeWidth="1.5" fill="none" /></svg></button>
      <button type="button" onClick={fullscreen} aria-label="Pantalla completa" title="Pantalla completa"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M2.5 6V2.5H6M10 2.5h3.5V6M13.5 10v3.5H10M6 13.5H2.5V10" stroke="currentColor" strokeWidth="1.5" fill="none" /></svg></button>
      <span aria-live="off" aria-label={`Zoom ${Math.round(zoom * 100)}%`}>{Math.round(zoom * 100)}%</span>
    </div>
    {snapshot && <div className="network-hint" aria-hidden="true">{layout === 'lineage' ? 'Color = familia · tamaño = riqueza · las líneas unen a cada hijo con su progenitor' : 'Derecha = más riqueza · cada franja es una generación'}</div>}
  </div>;
}

/** Linajes: posición fija calculada por el motor al nacer. Riqueza: x logarítmica, y por generación. */
function position(c: Citizen, layout: Layout, width: number, height: number, maxWealth: number, maxGeneration: number): [number, number] {
  if (layout === 'lineage') { const size = Math.min(width, height) * .98; return [width / 2 + (c.x - .5) * size, height / 2 + (c.y - .5) * size]; }
  const left = 56, right = 24, top = 30, bottom = 44; const bands = maxGeneration + 1;
  const wealth = c.alive ? c.money : 0;
  const x = left + (Math.log10(1 + Math.max(wealth, 0)) / Math.log10(1 + maxWealth)) * (width - left - right);
  const y = top + ((c.generation + .5 + (hash(c.id) - .5) * .7) / bands) * (height - top - bottom);
  return [x, y];
}

function drawAxis(scene: Scene, layout: Layout, snapshot: GenSnapshot | null) {
  scene.axis.clear(); for (const child of scene.labels.removeChildren()) child.destroy();
  if (layout !== 'wealth' || !snapshot) return;
  const width = scene.app.renderer.width / scene.app.renderer.resolution, height = scene.app.renderer.height / scene.app.renderer.resolution;
  const left = 56, right = 24, top = 30, bottom = 44; const maxGeneration = Math.max(1, ...snapshot.citizens.map(c => c.generation)); const bands = maxGeneration + 1;
  const maxWealth = Math.max(100, ...snapshot.citizens.filter(c => c.alive).map(c => c.money));
  const style = { fontFamily: 'Seravek, "Gill Sans Nova", Ubuntu, Calibri, system-ui, sans-serif', fontSize: 12, fill: 0x3f4a44 };
  const label = (text: string, x: number, y: number, anchorX = .5) => { const t = new Text({ text, style }); t.anchor.set(anchorX, .5); t.position.set(x, y); scene.labels.addChild(t); };
  for (let g = 0; g < bands; g++) {
    const y = top + (g / bands) * (height - top - bottom); if (g % 2 === 0) scene.axis.rect(left, y, width - left - right, (height - top - bottom) / bands).fill({ color: 0x18201d, alpha: .035 });
    label(g === 0 ? 'Fund.' : `Gen. ${g}`, left - 8, y + (height - top - bottom) / bands / 2, 1);
  }
  for (const value of [10, 100, 1_000, 10_000, 100_000]) {
    if (value > maxWealth * 1.2) break; const x = left + (Math.log10(1 + value) / Math.log10(1 + maxWealth)) * (width - left - right);
    scene.axis.moveTo(x, top).lineTo(x, height - bottom).stroke({ color: 0x18201d, width: 1, alpha: .1 });
    label(`${value.toLocaleString('es-MX')} u. m.`, x, height - bottom + 16);
  }
  const median = snapshot.metrics.median_wealth; const mx = left + (Math.log10(1 + Math.max(median, 0)) / Math.log10(1 + maxWealth)) * (width - left - right);
  scene.axis.moveTo(mx, top - 8).lineTo(mx, height - bottom).stroke({ color: BRASS, width: 1.5, alpha: .8 });
  label('mediana', mx, top - 16);
}

/** La persona seleccionada, sus ancestros y sus descendientes visibles. */
function familyOf(id: number, byId: Map<number, Citizen>) {
  const family = new Set<number>([id]); let cursor = byId.get(id)?.parent ?? null;
  while (cursor !== null && !family.has(cursor)) { family.add(cursor); cursor = byId.get(cursor)?.parent ?? null; }
  const queue = [id];
  while (queue.length) { const current = byId.get(queue.pop()!); for (const child of current?.children ?? []) if (!family.has(child)) { family.add(child); queue.push(child); } }
  return family;
}
