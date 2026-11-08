import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { GenerationsCanvas, lineageCss, type Layout } from './GenerationsCanvas';
import type { Citizen, FamilyView, GenConfig, GenEvent, GenSnapshot } from './generationsTypes';
import { Tip, api, money, num, pct, postJson } from './shared';
import './generations.css';

type Run = { id: string; config: GenConfig };
type Entry = { key: string; tick: number; text: string; person: number | null };
type Speed = 1 | 2 | 4;

const DEFAULT_CONFIG: GenConfig = { seed: 42, founders: 12, max_population: 300, inheritance_tax: 0, capital_return: 0.04 };
const SPEED: Record<Speed, { interval: number; count: number }> = { 1: { interval: 700, count: 1 }, 2: { interval: 350, count: 1 }, 4: { interval: 350, count: 2 } };
const STRATEGY_LABEL = { saver: 'ahorra', consumer: 'consume', entrepreneur: 'emprende' } as const;
const CAUSE_LABEL = { old_age: 'por vejez', poverty: 'porque no pudo cubrir su costo de vida durante 5 años' } as const;
const RULES = [
  'Cada año todos cumplen un año más.',
  'Entre los 18 y los 64 años se cobra un salario según la habilidad; el dinero ahorrado rinde según la estrategia.',
  'Cada adulto paga su costo de vida y el progenitor paga el de sus hijos menores.',
  'Quien tiene entre 20 y 45 años, al menos 60 u. m. y menos de 4 hijos puede tener un hijo; cerca del tope nacen menos.',
  'Al nacer, cada hijo recibe una dote del 25% del dinero de su progenitor y hereda su estrategia; su habilidad tiende al promedio.',
  'La probabilidad de morir crece con la edad; quien no cubre su costo de vida 5 años seguidos no puede sostenerse.',
  'Al morir, se cobra el impuesto a la herencia y el resto se reparte por igual entre los hijos vivos; sin hijos, todo va al fondo público.',
  'El fondo público se reparte por igual entre todas las personas vivas al final del año.',
];
const FIELDS: { key: keyof GenConfig; label: string; min: number; max: number; step: number; format: (v: number) => string; note: string }[] = [
  { key: 'founders', label: 'Fundadores', min: 2, max: 60, step: 1, format: v => `${v} familias`, note: 'Cada fundador inicia una familia con 100 u. m.' },
  { key: 'max_population', label: 'Población máxima', min: 20, max: 2000, step: 10, format: v => `${num(v)} personas`, note: 'Cerca del tope nacen menos hijos.' },
  { key: 'inheritance_tax', label: 'Impuesto a la herencia', min: 0, max: 1, step: .05, format: v => pct(v, 0), note: 'Lo recaudado se reparte por igual entre todos.' },
  { key: 'capital_return', label: 'Rendimiento del capital', min: 0, max: .1, step: .005, format: v => pct(v, 1), note: 'Lo que gana cada año el dinero acumulado.' },
];

export default function GenerationsView() {
  const [draft, setDraft] = useState<GenConfig>(DEFAULT_CONFIG); const [run, setRun] = useState<Run | null>(null); const [snapshot, setSnapshot] = useState<GenSnapshot | null>(null);
  const [stepEvents, setStepEvents] = useState<GenEvent[]>([]); const [log, setLog] = useState<Entry[]>([]); const [playing, setPlaying] = useState(false); const [speed, setSpeed] = useState<Speed>(1);
  const [layout, setLayout] = useState<Layout>('lineage'); const [selected, setSelected] = useState<number | null>(null); const [family, setFamily] = useState<FamilyView | null>(null);
  const [hoverLineage, setHoverLineage] = useState<number | null>(null); const [setup, setSetup] = useState(true); const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null); const [status, setStatus] = useState('');
  const names = useRef(new Map<number, { name: string; lineage: number }>()); const inFlight = useRef(false); const runRef = useRef(run); const snapRef = useRef(snapshot); runRef.current = run; snapRef.current = snapshot;

  const surname = (lineage: number) => snapshot?.lineages[lineage]?.surname ?? '';
  const fullName = (id: number) => { const known = names.current.get(id); return known ? `${known.name} ${surname(known.lineage)}` : 'una persona'; };

  function remember(snap: GenSnapshot) { for (const c of snap.citizens) names.current.set(c.id, { name: c.name, lineage: c.lineage }); }

  async function start() {
    if (busy) return; setBusy(true); setPlaying(false);
    try {
      const data = await api<{ id: string; snapshot: GenSnapshot }>('/api/generations', postJson({ config: draft }));
      names.current.clear(); remember(data.snapshot);
      setRun({ id: data.id, config: data.snapshot.config }); setSnapshot(data.snapshot); setStepEvents([]); setSelected(null); setFamily(null); setSetup(false); setError(null);
      setLog([{ key: 'start', tick: 0, text: `Año 0 · ${data.snapshot.config.founders} familias fundadoras empiezan con 100 u. m. cada una.`, person: null }]);
      setStatus('Sociedad creada. Pulsa Reproducir o Espacio para que pasen los años.');
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); } finally { setBusy(false); }
  }

  /** Avanza; si el servidor perdió la corrida (reinicio), la recrea en el mismo año porque el motor es determinista. */
  async function advance(count: number) {
    const current = runRef.current; const snap = snapRef.current; if (!current || !snap || inFlight.current || snap.finished) return; inFlight.current = true;
    try {
      let id = current.id; let data: { snapshot: GenSnapshot; events: GenEvent[] };
      try { data = await api(`/api/generations/${id}/step`, postJson({ count })); }
      catch (e) {
        if (!(e instanceof Error && e.message.includes('(404)'))) throw e;
        const restored = await api<{ id: string }>('/api/generations', postJson({ config: current.config, advance_to: snap.tick }));
        id = restored.id; setRun({ id, config: current.config });
        data = await api(`/api/generations/${id}/step`, postJson({ count }));
      }
      remember(data.snapshot); setSnapshot(data.snapshot); setStepEvents(data.events); setError(null);
      setLog(entries => [...describe(data.events, fullName, surname), ...entries].slice(0, 60));
      const m = data.snapshot.metrics; const births = data.events.filter(e => e.kind === 'birth').length, deaths = data.events.filter(e => e.kind === 'death').length;
      setStatus(data.snapshot.finished ? (m.population === 0 ? `Año ${m.tick}: la sociedad se extinguió.` : `Año ${m.tick}: se alcanzó el final de la simulación.`) : `Año ${m.tick}: ${births} ${births === 1 ? 'nacimiento' : 'nacimientos'}, ${deaths} ${deaths === 1 ? 'muerte' : 'muertes'}.`);
      if (data.snapshot.finished) setPlaying(false);
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); setPlaying(false); } finally { inFlight.current = false; }
  }

  async function reset() {
    if (!run) return; setPlaying(false);
    try {
      const snap = await api<GenSnapshot>(`/api/generations/${run.id}/reset`, { method: 'POST' });
      names.current.clear(); remember(snap); setSnapshot(snap); setStepEvents([]); setSelected(null); setFamily(null);
      setLog([{ key: `reset-${Date.now()}`, tick: 0, text: 'Año 0 · La misma sociedad vuelve a empezar con la misma semilla.', person: null }]); setStatus('Sociedad reiniciada en el año 0.');
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); }
  }

  useEffect(() => {
    if (!playing) return; const { interval, count } = SPEED[speed]; let timer = 0; let alive = true;
    const tick = async () => { await advance(count); if (alive) timer = window.setTimeout(tick, interval); };
    timer = window.setTimeout(tick, 0); return () => { alive = false; window.clearTimeout(timer); };
  }, [playing, speed]);

  useEffect(() => {
    if (!run || selected === null) { setFamily(null); return; } let cancelled = false;
    api<FamilyView>(`/api/generations/${run.id}/person/${selected}`).then(view => { if (!cancelled) setFamily(view); }).catch(() => { if (!cancelled) setFamily(null); });
    return () => { cancelled = true; };
  }, [run?.id, selected, snapshot?.tick]);

  const keys = useRef<{ toggle: () => void; advance: (n: number) => void; flip: () => void }>({ toggle: () => {}, advance: () => {}, flip: () => {} });
  keys.current = { toggle: () => { if (snapshot && !snapshot.finished && !setup) setPlaying(p => !p); }, advance: (n: number) => { if (!playing && !setup) void advance(n); }, flip: () => setLayout(l => (l === 'lineage' ? 'wealth' : 'lineage')) };
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement; if (event.metaKey || event.ctrlKey || event.altKey || target.closest('input,select,textarea,button,summary,a,[contenteditable]')) return;
      if (event.code === 'Space') { event.preventDefault(); keys.current.toggle(); }
      else if (event.key === 'ArrowRight') { event.preventDefault(); keys.current.advance(event.shiftKey ? 10 : 1); }
      else if (event.key.toLowerCase() === 'l') keys.current.flip();
      else if (event.key === 'Escape') setSelected(null);
    };
    window.addEventListener('keydown', onKey); return () => window.removeEventListener('keydown', onKey);
  }, []);

  const m = snapshot?.metrics; const giniHistory = useMemo(() => snapshot?.metric_history.map(h => h.gini) ?? [], [snapshot]);
  const lineages = useMemo(() => (snapshot?.lineages ?? []).slice().sort((a, b) => Number(b.alive > 0) - Number(a.alive > 0) || b.share - a.share || (b.extinct_tick ?? 0) - (a.extinct_tick ?? 0)), [snapshot]);
  const duration = Math.min(450, SPEED[speed].interval * .8);
  const canStep = !!snapshot && !snapshot.finished && !playing && !setup;

  return <main className="gen">
    <header className="gen-bar">
      <div className="gen-title"><h1>Economía viva</h1><p>Una sociedad pequeña que nace, hereda y se transforma, año por año.</p></div>
      <div className="segmented" role="group" aria-label="Forma del mapa">
        <button type="button" aria-pressed={layout === 'lineage'} onClick={() => setLayout('lineage')}>Familias</button>
        <button type="button" aria-pressed={layout === 'wealth'} onClick={() => setLayout('wealth')}>Riqueza</button>
      </div>
      <div className="gen-bar-actions">
        <button type="button" className="btn secondary" onClick={() => { setPlaying(false); setSetup(true); }} disabled={setup}>Nueva sociedad</button>
        <a className="btn secondary" href="#laboratorio">Laboratorio completo</a>
      </div>
    </header>

    {error && <div className="error-banner" role="alert"><p>{error}</p><button type="button" className="btn small secondary" onClick={() => setError(null)}>Cerrar</button></div>}

    <div className="gen-grid">
      <section className="gen-stage" aria-label="Sociedad">
        <GenerationsCanvas snapshot={snapshot} events={stepEvents} layout={layout} selected={selected} highlightLineage={hoverLineage} duration={duration} onSelect={setSelected}>
          {setup && <SetupCard draft={draft} setDraft={setDraft} busy={busy} onStart={start} onCancel={snapshot ? () => setSetup(false) : undefined} />}
          {!setup && family && <Inspector family={family} onSelect={setSelected} onClose={() => setSelected(null)} />}
        </GenerationsCanvas>
        <div className="gen-transport">
          <button type="button" className="btn" disabled={!snapshot || snapshot.finished || setup} onClick={() => setPlaying(p => !p)} aria-keyshortcuts="Space">
            {playing ? <svg className="icon" viewBox="0 0 16 16" aria-hidden="true"><path d="M4.5 3h2.4v10H4.5zM9.1 3h2.4v10H9.1z" fill="currentColor" /></svg> : <svg className="icon" viewBox="0 0 16 16" aria-hidden="true"><path d="M4.5 3.2v9.6L12.5 8z" fill="currentColor" /></svg>}
            {playing ? 'Pausar' : 'Reproducir'}
          </button>
          <button type="button" className="btn secondary" disabled={!canStep} onClick={() => advance(1)} aria-keyshortcuts="ArrowRight">+1 año</button>
          <button type="button" className="btn secondary" disabled={!canStep} onClick={() => advance(10)} aria-keyshortcuts="Shift+ArrowRight">+10 años</button>
          <div className="segmented small" role="group" aria-label="Velocidad">
            {([1, 2, 4] as Speed[]).map(s => <button key={s} type="button" aria-pressed={speed === s} onClick={() => setSpeed(s)}>{s}×</button>)}
          </div>
          <strong className="gen-year">Año {snapshot?.tick ?? 0}</strong>
          <button type="button" className="btn small secondary" disabled={!run || setup} onClick={reset}>Reiniciar</button>
          <span className="gen-status" aria-live="polite">{status}</span>
        </div>
      </section>

      <aside className="gen-rail" aria-label="Resumen de la sociedad">
        <dl className="gen-figures">
          <div><dt><Tip term="Población máxima" label="Población" /></dt><dd><strong>{num(m?.population)}</strong><span> de {num(snapshot?.config.max_population ?? draft.max_population)}</span></dd></div>
          <div><dt><Tip term="Gini" label="Desigualdad (Gini)" /></dt><dd><strong>{m ? num(m.gini, 2) : '—'}</strong><Trend values={giniHistory} /></dd></div>
          <div><dt><Tip term="Riqueza heredada" /></dt><dd><strong>{pct(m?.inherited_share, 0)}</strong><span> del dinero de los vivos</span></dd></div>
        </dl>

        <section className="gen-families" aria-labelledby="families-title">
          <h2 id="families-title"><Tip term="Linaje" label="Familias" /></h2>
          {lineages.length ? <ul>{lineages.map(l => <li key={l.id} className={l.alive ? '' : 'extinct'} onMouseEnter={() => setHoverLineage(l.id)} onMouseLeave={() => setHoverLineage(null)}>
            <button type="button" onFocus={() => setHoverLineage(l.id)} onBlur={() => setHoverLineage(null)} onClick={() => setHoverLineage(h => (h === l.id ? null : l.id))} aria-label={`Familia ${l.surname}: ${l.alive ? `${l.alive} ${l.alive === 1 ? 'vivo' : 'vivos'}, ${pct(l.share, 0)} de la riqueza` : `extinta en el año ${l.extinct_tick}`}. Resaltar en el mapa.`}>
              <i style={{ background: l.alive ? lineageCss(l.id) : 'var(--line-strong)' }} aria-hidden="true" />
              <span className="family-name">{l.surname}</span>
              <span className="family-meta">{l.alive ? `${l.alive} ${l.alive === 1 ? 'vivo' : 'vivos'}` : `extinta · año ${l.extinct_tick}`}</span>
              <span className="family-bar" aria-hidden="true"><b style={{ width: `${Math.max(l.share * 100, l.alive ? 1 : 0)}%`, background: lineageCss(l.id) }} /></span>
              <span className="family-share">{l.alive ? pct(l.share, 0) : ''}</span>
            </button>
          </li>)}</ul> : <p className="muted">Las familias aparecen al crear la sociedad.</p>}
        </section>

        <section className="gen-log" aria-labelledby="log-title">
          <h2 id="log-title">Qué pasó</h2>
          {log.length ? <ol>{log.slice(0, 8).map(entry => <li key={entry.key}>
            {entry.person !== null ? <button type="button" onClick={() => setSelected(entry.person)}>{entry.text}</button> : <span>{entry.text}</span>}
          </li>)}</ol> : <p className="muted">Aquí se cuentan los nacimientos, muertes y herencias de cada año.</p>}
        </section>
      </aside>
    </div>
  </main>;
}

function SetupCard({ draft, setDraft, busy, onStart, onCancel }: { draft: GenConfig; setDraft: (c: GenConfig) => void; busy: boolean; onStart: () => void; onCancel?: () => void }) {
  const seedId = useId();
  return <form className="gen-overlay gen-setup" onSubmit={e => { e.preventDefault(); onStart(); }}>
    <h2>¿Cómo empieza tu sociedad?</h2>
    <p>Unas pocas familias empiezan con el mismo dinero. Cada año trabajan, ahorran, tienen hijos y mueren. Observa quién acaba con la riqueza y por qué.</p>
    <div className="setup-fields">
      {FIELDS.map(field => <SetupSlider key={field.key} field={field} value={draft[field.key] as number} onChange={value => setDraft({ ...draft, [field.key]: value })} />)}
      <div className="setup-seed">
        <label htmlFor={seedId}><Tip term="Semilla" /></label>
        <input id={seedId} type="number" min={0} value={draft.seed} onChange={e => setDraft({ ...draft, seed: Math.max(0, Math.floor(Number(e.target.value) || 0)) })} />
        <button type="button" className="btn small secondary" onClick={() => setDraft({ ...draft, seed: Math.floor(Math.random() * 1_000_000) })}>Otra semilla</button>
      </div>
    </div>
    <details className="setup-rules"><summary>Reglas del juego</summary><ol>{RULES.map(rule => <li key={rule}>{rule}</li>)}</ol></details>
    <div className="setup-actions">
      <button type="submit" className="btn" disabled={busy}>{busy ? 'Creando…' : 'Comenzar'}</button>
      {onCancel && <button type="button" className="btn secondary" onClick={onCancel}>Volver a la sociedad actual</button>}
    </div>
  </form>;
}

function SetupSlider({ field, value, onChange }: { field: typeof FIELDS[number]; value: number; onChange: (v: number) => void }) {
  const id = useId();
  return <div className="slider">
    <div className="slider-head"><label htmlFor={id}><Tip term={field.label} /></label><output htmlFor={id}>{field.format(value)}</output></div>
    <input id={id} type="range" min={field.min} max={field.max} step={field.step} value={value} onChange={e => onChange(Number(e.target.value))} aria-describedby={`${id}-note`} />
    <small id={`${id}-note`}>{field.note}</small>
  </div>;
}

function Inspector({ family, onSelect, onClose }: { family: FamilyView; onSelect: (id: number) => void; onClose: () => void }) {
  const p = family.person; const received = Math.max(p.received, 0), earned = Math.max(p.earned, 0); const total = received + earned;
  const person = (c: Citizen) => <button type="button" className="person-link" onClick={() => onSelect(c.id)}><i style={{ background: c.alive ? lineageCss(c.lineage) : 'var(--line-strong)' }} aria-hidden="true" />{c.name}{c.alive ? '' : ' (fallecido)'}</button>;
  return <section className="gen-overlay gen-inspector" aria-label={`Ficha de ${p.name} ${family.surname}`}>
    <div className="inspector-head">
      <h2><i style={{ background: lineageCss(p.lineage) }} aria-hidden="true" />{p.name} {family.surname}</h2>
      <button type="button" className="btn small secondary" onClick={onClose} aria-label="Cerrar ficha">Cerrar</button>
    </div>
    <p className="inspector-facts">{p.alive ? `${p.age} ${p.age === 1 ? 'año' : 'años'}` : `Murió a los ${p.age} años ${p.cause ? CAUSE_LABEL[p.cause] : ''}`} · <Tip term="Generación" label={p.generation === 0 ? 'fundador' : `generación ${p.generation}`} /> · {STRATEGY_LABEL[p.strategy]}</p>
    {p.alive && <p className="inspector-money"><strong>{money(p.money)}</strong>{p.delta !== 0 && <span className={p.delta > 0 ? 'up' : 'down'}>{p.delta > 0 ? '+' : '−'}{money(Math.abs(p.delta))} este año</span>}</p>}
    <div className="origin">
      <span className="origin-title">De dónde vino su dinero</span>
      <div className="origin-bar" role="img" aria-label={`Recibido ${pct(total ? received / total : 0, 0)}, ganado ${pct(total ? earned / total : 0, 0)}`}>
        <b className="received" style={{ width: `${total ? (received / total) * 100 : 0}%` }} /><b className="earned" style={{ width: `${total ? (earned / total) * 100 : 0}%` }} />
      </div>
      <dl><div><dt><i className="received" aria-hidden="true" /><Tip term="Herencia" label="Recibido" /></dt><dd>{money(received)}</dd></div><div><dt><i className="earned" aria-hidden="true" />Ganado</dt><dd>{money(earned)}</dd></div></dl>
      <small>Recibido incluye dote, herencias y fondo público. Ganado incluye salarios y rendimientos, antes del costo de vida.</small>
    </div>
    <dl className="inspector-family">
      <div><dt>Progenitor</dt><dd>{family.parent ? person(family.parent) : 'Fundó la familia'}</dd></div>
      <div><dt>Hijos</dt><dd>{family.children.length ? family.children.map(c => <span key={c.id}>{person(c)}</span>) : 'Sin hijos'}</dd></div>
      {family.ancestors.length > 1 && <div><dt>Linaje</dt><dd className="lineage-chain">{family.ancestors.slice().reverse().map(c => <span key={c.id}>{person(c)}</span>)}</dd></div>}
    </dl>
  </section>;
}

function Trend({ values }: { values: number[] }) {
  if (values.length < 2) return null;
  const max = Math.max(...values, .01); const points = values.map((v, i) => `${(i / (values.length - 1)) * 100},${30 - (v / max) * 26}`).join(' ');
  return <svg className="trend" viewBox="0 0 100 32" preserveAspectRatio="none" role="img" aria-label={`Evolución del Gini: de ${num(values[0], 2)} a ${num(values[values.length - 1], 2)}`}><polyline points={points} fill="none" stroke="currentColor" strokeWidth="1.5" vectorEffect="non-scaling-stroke" /></svg>;
}

/** Convierte los eventos de uno o varios años en frases cortas, de la más reciente a la más antigua. */
function describe(events: GenEvent[], fullName: (id: number) => string, surname: (lineage: number) => string): Entry[] {
  const ticks = [...new Set(events.map(e => e.tick))].sort((a, b) => b - a); const entries: Entry[] = [];
  for (const tick of ticks) {
    const year = events.filter(e => e.tick === tick);
    for (const e of year) {
      if (e.kind === 'death') {
        const inheritance = year.find(x => x.kind === 'inheritance' && x.from === e.id);
        let text = e.cause === 'poverty' ? `Año ${tick} · ${fullName(e.id)} no pudo sostenerse: llevaba 5 años sin cubrir su costo de vida.` : `Año ${tick} · Muere ${fullName(e.id)} a los ${e.age} años.`;
        if (inheritance?.kind === 'inheritance') {
          text += inheritance.heirs.length
            ? ` ${inheritance.heirs.length === 1 ? 'Su hijo hereda' : `Sus ${inheritance.heirs.length} hijos heredan`} ${money(inheritance.each)}${inheritance.heirs.length > 1 ? ' cada uno' : ''}${inheritance.tax > 0 ? ` (impuesto: ${money(inheritance.tax)})` : ''}`
            : ` Sin hijos vivos: ${money(inheritance.to_state)} pasan al fondo público.`;
        }
        entries.push({ key: `d-${e.id}`, tick, text, person: inheritance?.kind === 'inheritance' && inheritance.heirs.length ? inheritance.heirs[0] : null });
      } else if (e.kind === 'extinction') entries.push({ key: `x-${e.lineage}`, tick, text: `Año ${tick} · La familia ${surname(e.lineage)} se extingue.`, person: null });
      else if (e.kind === 'collapse') entries.push({ key: `c-${tick}`, tick, text: `Año ${tick} · No queda nadie vivo: la sociedad se extinguió.`, person: null });
    }
    const births = year.filter(e => e.kind === 'birth');
    if (births.length === 1) { const b = births[0]; entries.push({ key: `b-${b.child}`, tick, text: `Año ${tick} · Nace ${fullName(b.child)} y recibe de ${fullName(b.parent).split(' ')[0]} una dote de ${money(b.dowry)}`, person: b.child }); }
    else if (births.length > 1) { const shown = births.slice(0, 2).map(b => fullName(b.child)); entries.push({ key: `b-${births[0].child}`, tick, text: `Año ${tick} · Nacen ${births.length} personas: ${shown.join(', ')}${births.length > 2 ? ` y ${births.length - 2} más` : ''}.`, person: births[0].child }); }
  }
  return entries;
}
