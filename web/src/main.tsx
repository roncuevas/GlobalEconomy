import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { createRoot } from 'react-dom/client';
import { create } from 'zustand';
import { NetworkCanvas } from './NetworkCanvas';
import './style.css';
import './risk.css';

type Metrics = { gdp:number; gini:number; unemployment:number; inflation:number; tax_revenue:number; trade_volume:number; alerts:number; mean_wealth:number; money_supply:number; interest_rate:number; top_10_wealth_share:number; money_created_this_tick:number; cumulative_money_created:number; government_debt:number; private_debt:number; debt_created_this_tick:number; monetary_regime:string; wealth_shock:number; adverse_events:number; open_risks:number; countermeasures:number; systemic_risk:number; security_level:number };
type Policy = { income_tax_rate:number; transfer_per_person:number; base_interest_rate:number; liquidity_injection:number };
type MonetaryPolicy = { apply_tick:number; era:string; name:string; description:string; base_interest_rate:number; liquidity_injection:number; government_debt_change:number; private_credit_creation:number; wealth_shock:number; catastrophe:boolean; optional:boolean };
type MoneyEvent = { tick:number; kind:string; amount:number; reason:string };
type Agent = { id:number; kind:string; region:number; x:number; y:number; size:number; risk_score:number; employed:boolean; wealth:number; wealth_delta:number };
type Edge = { source:number; target:number; kind:string; weight:number };
type MetricFrame = { tick:number; gdp:number; gini:number; unemployment:number; inflation:number; money_supply:number; government_debt:number; private_debt:number; mean_wealth:number; systemic_risk:number; security_level:number };
type RiskEvent = { id:number; policy_tick:number; policy_name:string; category:string; name:string; description:string; detection_reason:string; severity:number; detected_at:number; countermeasure_name:string; countermeasure_description:string; countermeasure_applied:boolean; before:MetricFrame; after_policy:MetricFrame|null; after_countermeasure:MetricFrame|null };
type Snapshot = { tick:number; agents:Agent[]; edges:Edge[]; metrics:Metrics; policy:Policy; monetary_history:MonetaryPolicy[]; money_events:MoneyEvent[]; active_policy:MonetaryPolicy|null; next_policy:MonetaryPolicy|null; policy_mode:'manual'|'automatic'; risk_events:RiskEvent[] };
type Store = { snapshot:Snapshot|null; running:boolean; setSnapshot:(s:Snapshot)=>void; setRunning:(v:boolean)=>void };
const useStore = create<Store>((set)=>({snapshot:null,running:false,setSnapshot:(snapshot)=>set({snapshot}),setRunning:(running)=>set({running})}));
const API = 'http://localhost:8080';
const TERM_HELP: Record<string,string> = {
  'PIB':'Producto Interno Bruto: valor agregado de la actividad económica simulada.',
  'u. m.':'Unidades monetarias sintéticas del laboratorio; no representan una moneda real.',
  'Manual':'El usuario decide cuándo activar la siguiente política.',
  'Automática':'El motor activa las políticas al llegar a sus ticks programados.',
  'opcional':'Evento que solo aparece cuando se activan las catástrofes opcionales.',
  'Tick':'Unidad temporal de la simulación. Cada tick representa una semana económica.',
  'PIB simulado':'Producción e ingresos generados por las personas durante el tick.',
  'Oferta monetaria':'Dinero disponible en las cuentas y bolsillos de los agentes.',
  'Creado este tick':'Dinero nuevo creado por liquidez o crédito durante el tick actual.',
  'Deuda pública':'Obligaciones acumuladas del gobierno frente al sistema económico.',
  'Deuda privada':'Obligaciones acumuladas de personas y empresas por crédito.',
  'Gini':'Índice de desigualdad: 0 significa igualdad total y 1 concentración máxima.',
  'Top 10% riqueza':'Porción de la riqueza total controlada por el diez por ciento más rico.',
  'Catástrofes opcionales':'Activa shocks sintéticos como burbujas, guerras, pandemias o crisis cambiarias.',
  'Nueva corrida':'Crea una simulación nueva usando la configuración actual y una semilla reproducible.',
  'Play':'Inicia la evolución automática de la corrida.',
  'Pausar':'Detiene temporalmente la evolución y conserva el estado actual.',
  'Step':'Avanza exactamente un tick cuando la corrida está pausada.',
  'Vista rápida':'Controles para reducir visualmente los nodos mostrados sin alterar la economía.',
  'Tipo':'Filtra por clase de entidad económica.',
  'Dinero':'Filtra por patrimonio relativo a la media de las personas.',
  'Solo riesgo alto':'Muestra únicamente personas cuyo riesgo individual supera el umbral alto.',
  'Política':'Elige si las políticas históricas avanzan automáticamente o mediante intervención manual.',
  'Aplicar siguiente':'Activa la siguiente política monetaria disponible sin avanzar el reloj.',
  'Aplicar contramedida':'Responde a la alerta abierta y registra el efecto antes/después.',
  'Red económica 2D':'Mapa de entidades y relaciones económicas en dos dimensiones.',
  'GPU':'Procesamiento gráfico usado para dibujar muchos nodos con fluidez.',
  'Persona':'Agente que recibe ingresos, consume, trabaja y acumula patrimonio.',
  'Mercado regional':'Punto de encuentro de oferta, demanda y comercio de una región.',
  'Banco central':'Entidad que modifica tasas, liquidez y reglas monetarias.',
  'Comercio':'Conexión de una persona con su mercado regional.',
  'Flujo monetario':'Conexión que representa creación o movimiento institucional de dinero.',
  'Crecimiento':'Borde verde que indica aumento reciente de patrimonio.',
  'Política monetaria':'Decisiones que cambian tasas, liquidez, crédito y régimen monetario.',
  'Fase vigente':'Política histórica actualmente aplicada al motor.',
  'Tasa de interés':'Costo del crédito y rendimiento base de los saldos monetarios.',
  'Inyección de liquidez':'Cantidad de dinero nuevo que entra al sistema desde la autoridad monetaria.',
  'Creado acumulado':'Total de dinero creado desde el inicio de la corrida.',
  'Deuda creada este tick':'Variación de deuda generada por la política aplicada en este tick.',
  'Evolución del dinero':'Línea temporal de los cambios históricos y políticos del sistema monetario.',
  'Hitos históricos':'Eventos de referencia que marcan cambios en la forma de crear, respaldar o controlar dinero.',
  'Nodo seleccionado':'Entidad cuyo estado detallado se está inspeccionando.',
  'Patrimonio':'Riqueza neta: dinero y activos menos deuda.',
  'Variación':'Cambio de patrimonio desde el tick anterior.',
  'Región':'Zona geográfica sintética donde vive o opera la entidad.',
  'Estado':'Situación laboral actual de la persona.',
  'Riesgo':'Puntuación sintética de vulnerabilidad o comportamiento anómalo.',
  'Estadísticas en tiempo real':'Series recientes de indicadores para observar la evolución de la corrida.',
  'Desempleo':'Proporción de personas que no tienen empleo en el tick.',
  'Alerta detectada':'Evento adverso identificado antes o durante una política.',
  'Alerta resuelta':'Alerta a la que ya se aplicó una contramedida.',
  'Riesgo sistémico':'Presión agregada sobre la estabilidad del sistema económico.',
  'Seguridad monetaria':'Nivel sintético de protección de la autenticidad e integridad del dinero.',
  'Detección':'Señal observable que explica por qué se abrió la alerta.',
  'Contramedida propuesta':'Respuesta defensiva sugerida por el modelo para reducir el riesgo.',
  'Historial de alertas':'Registro de alertas, su estado y sus métricas antes y después.',
  'Contramedida aplicada':'Indica que la respuesta defensiva ya fue ejecutada.',
  'Abierta':'La alerta todavía no tiene una contramedida registrada.',
  'Génesis monetaria':'Inicio del experimento: todos reciben la misma dotación monetaria.',
  'commodity':'Dinero basado inicialmente en una dotación material o recurso.',
  'convertible':'Dinero que puede canjearse por un activo de respaldo bajo reglas explícitas.',
  'credit':'Dinero bancario creado al conceder préstamos.',
  'fiat':'Dinero cuyo valor depende de aceptación, confianza y reglas institucionales.',
};

function Tip({term, children}: {term:string; children?:ReactNode}) {
  const explanation = TERM_HELP[term] ?? term;
  return <span className="term-tip" tabIndex={0} title={explanation} aria-label={`${term}: ${explanation}`}>{children ?? term}<span className="term-tip-bubble" role="tooltip">{explanation}</span></span>;
}

function App(){
  const {snapshot,running,setSnapshot,setRunning}=useStore();
  const [id,setId]=useState(''); const [history,setHistory]=useState<Snapshot[]>([]); const [selected,setSelected]=useState<Agent|null>(null); const [catastrophes,setCatastrophes]=useState(true); const [kindFilter,setKindFilter]=useState('all'); const [wealthFilter,setWealthFilter]=useState('all'); const [riskOnly,setRiskOnly]=useState(false); const socket=useRef<WebSocket|null>(null);
  const record=(s:Snapshot)=>{setSnapshot(s);setHistory(h=>[...h.slice(-39),s]);};
  async function createRun(){ const r=await fetch(`${API}/api/runs`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({seed:42,model_version:'0.4.0',agent_count:1000,regions:8,ticks:1000,historical_mode:true,catastrophes,policy_mode:'manual',policy_config:{income_tax_rate:.18,transfer_per_person:20,base_interest_rate:.04,liquidity_injection:0},scenario_config:{food_price:12,wage:100,initial_wealth:2500}})}); const data=await r.json(); setId(data.run_id); const s=await fetch(`${API}/api/runs/${data.run_id}/snapshot`); const snap=await s.json(); setHistory([snap]);setSnapshot(snap); }
  async function step(){ if(!id) return; const r=await fetch(`${API}/api/runs/${id}/step`,{method:'POST'}); record(await r.json()); }
  async function updatePolicy(key:keyof Policy,value:number){ if(!id||!snapshot) return; const policy={...snapshot.policy,[key]:value}; const r=await fetch(`${API}/api/runs/${id}/policy`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(policy)}); record(await r.json()); }
  async function applyNextPolicy(){ if(!id||running) return; const r=await fetch(`${API}/api/runs/${id}/policy/next`,{method:'POST'}); if(r.ok)record(await r.json()); }
  async function updatePolicyMode(mode:'manual'|'automatic'){ if(!id) return; const r=await fetch(`${API}/api/runs/${id}/policy-mode`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({mode})}); if(r.ok)record(await r.json()); }
  async function applyCountermeasure(){ if(!id) return; const r=await fetch(`${API}/api/runs/${id}/risk/countermeasure`,{method:'POST'}); if(r.ok)record(await r.json()); }
  function toggle(){ if(!id) return; const next=!running; fetch(`${API}/api/runs/${id}/${next?'start':'pause'}`,{method:'POST'}); setRunning(next); if(next){ socket.current?.close(); socket.current=new WebSocket(`${API.replace('http','ws')}/api/runs/${id}/stream`); socket.current.onmessage=(e:MessageEvent<string>)=>{const m=JSON.parse(e.data);record(m.data);}; } else socket.current?.close(); }
  useEffect(()=>()=>socket.current?.close(),[]);
  const filteredSnapshot=useMemo(()=>{if(!snapshot)return null;const people=snapshot.agents.filter(a=>a.kind==='person');const mean=people.reduce((sum,a)=>sum+a.wealth,0)/Math.max(people.length,1);const agents=snapshot.agents.filter(a=>{if(kindFilter!=='all'&&a.kind!==kindFilter)return false;if(wealthFilter==='low'&&a.wealth>mean*.8)return false;if(wealthFilter==='middle'&&(a.wealth<=mean*.8||a.wealth>=mean*1.3))return false;if(wealthFilter==='high'&&a.wealth<mean*1.3)return false;if(riskOnly&&a.risk_score<.8)return false;return true;});const ids=new Set(agents.map(a=>a.id));return {...snapshot,agents,edges:snapshot.edges.filter(e=>ids.has(e.source)&&ids.has(e.target)).slice(0,2500)};},[snapshot,kindFilter,wealthFilter,riskOnly]);
  const m=snapshot?.metrics; const p=snapshot?.policy; const nodes=filteredSnapshot?.agents??[]; const visibleEdges=filteredSnapshot?.edges??[];
  const activePhase=snapshot?.active_policy; const nextPhase=snapshot?.next_policy; const activePolicyTick=activePhase?.apply_tick??0; const openRisk=[...(snapshot?.risk_events??[])].reverse().find(event=>!event.countermeasure_applied); const latestRisk=snapshot?.risk_events?.[snapshot.risk_events.length-1]; const riskForPanel=openRisk??latestRisk;
  return <main><header><div><span className="eyebrow">GLOBAL ECONOMY / NETWORK LAB</span><h1>Economía viva</h1><p>Mapa bidimensional de nodos interconectados. La riqueza cambia el tamaño; las relaciones muestran comercio y mercados.</p></div><div className="controls"><label className="toggle" title={TERM_HELP['Catástrofes opcionales']}><input type="checkbox" checked={catastrophes} onChange={e=>setCatastrophes(e.target.checked)}/><span><Tip term="Catástrofes opcionales"/></span></label><button title={TERM_HELP['Nueva corrida']} onClick={createRun}>Nueva corrida</button><button title={TERM_HELP[running?'Pausar':'Play']} disabled={!id} onClick={toggle}>{running?'Pausar':'Play'}</button><button title={TERM_HELP.Step} disabled={!id||running} onClick={step}>Step</button></div></header>
    <section className="stats"><Stat label="Tick" value={snapshot?.tick??'—'}/><Stat label="PIB simulado" value={m?m.gdp.toFixed(0):'—'}/><Stat label="Oferta monetaria" value={m?compact(m.money_supply):'—'}/><Stat label="Creado este tick" value={m?compact(m.money_created_this_tick):'—'}/><Stat label="Deuda pública" value={m?compact(m.government_debt):'—'}/><Stat label="Deuda privada" value={m?compact(m.private_debt):'—'}/><Stat label="Gini" value={m?m.gini.toFixed(3):'—'}/><Stat label="Top 10% riqueza" value={m?`${(m.top_10_wealth_share*100).toFixed(1)}%`:'—'}/></section>
    <section className="filter-strip"><strong><Tip term="Vista rápida"/></strong><label title={TERM_HELP.Tipo}>Tipo<select value={kindFilter} onChange={e=>setKindFilter(e.target.value)}><option value="all">Todos los nodos</option><option value="person">Solo personas</option><option value="market">Solo mercados</option><option value="central_bank">Solo bancos</option></select></label><label title={TERM_HELP.Dinero}>Dinero<select value={wealthFilter} onChange={e=>setWealthFilter(e.target.value)}><option value="all">Cualquier patrimonio</option><option value="low">Bajo</option><option value="middle">Medio</option><option value="high">Alto</option></select></label><label className="toggle" title={TERM_HELP['Solo riesgo alto']}><input type="checkbox" checked={riskOnly} onChange={e=>setRiskOnly(e.target.checked)}/><span>Solo riesgo alto</span></label><label title={TERM_HELP.Política}>Política<select value={snapshot?.policy_mode??'manual'} onChange={e=>updatePolicyMode(e.target.value as 'manual'|'automatic')}><option value="manual">Manual</option><option value="automatic">Automática</option></select></label><button className="policy-next" title={TERM_HELP['Aplicar siguiente']} disabled={!id||running||!nextPhase} onClick={applyNextPolicy}>Aplicar siguiente{nextPhase?` · ${nextPhase.name}`:''}</button><button className="countermeasure" title={TERM_HELP['Aplicar contramedida']} disabled={!id||!openRisk} onClick={applyCountermeasure}>Aplicar contramedida{openRisk?` · ${openRisk.countermeasure_name}`:''}</button><small title="Cantidad de nodos y conexiones que cumplen los filtros actuales">{nodes.length} nodos visibles · {visibleEdges.length} conexiones</small></section>
    {riskForPanel?<section className="risk-panel"><div><small title={TERM_HELP[openRisk?'Alerta detectada':'Alerta resuelta']}>{openRisk?'ALERTA DETECTADA':'ALERTA RESUELTA'} · tick {riskForPanel.detected_at} · severidad {(riskForPanel.severity*100).toFixed(0)}%</small><h2 title={riskForPanel.description}>{riskForPanel.name}</h2><p>{riskForPanel.description}</p><p className="muted"><Tip term="Detección"/>: {riskForPanel.detection_reason}</p><p className="muted"><Tip term="Contramedida propuesta"/>: {riskForPanel.countermeasure_description}</p></div><div className="risk-stat"><span><Tip term="Riesgo sistémico"/></span><strong>{((m?.systemic_risk??0)*100).toFixed(0)}%</strong><span><Tip term="Seguridad monetaria"/></span><strong>{((m?.security_level??0)*100).toFixed(0)}%</strong></div><details open={!openRisk}><summary><Tip term="Historial de alertas"/> ({snapshot?.risk_events.length??0})</summary>{(snapshot?.risk_events??[]).slice().reverse().map(event=><div className="risk-row" key={event.id}><strong title={event.description}>{event.name}</strong><span title={event.countermeasure_applied?TERM_HELP['Contramedida aplicada']:TERM_HELP.Abierta}>tick {event.detected_at} · {event.countermeasure_applied?'contramedida aplicada':'abierta'}</span><small>Antes: riesgo {(event.before.systemic_risk*100).toFixed(0)}% · Después: {event.after_countermeasure?`riesgo ${(event.after_countermeasure.systemic_risk*100).toFixed(0)}%`:'pendiente'}</small></div>)}</details></section>:<section className="risk-panel quiet"><span>Sin riesgos monetarios abiertos detectados.</span><small>{m?.countermeasures??0} contramedidas aplicadas · {m?.adverse_events??0} eventos registrados</small></section>}
    <section className="workspace"><div className="world"><div className="map-title"><span><Tip term="Red económica 2D"/> · <Tip term="GPU"/></span><small title="Cantidad de personas, nodos y conexiones que cumplen los filtros actuales">{nodes.filter(n=>n.kind==='person').length} personas · {nodes.length} nodos · {visibleEdges.length} conexiones</small></div><NetworkCanvas snapshot={filteredSnapshot} onSelect={setSelected}/><div className="legend"><span><i className="dot person"/><Tip term="Persona"/></span><span><i className="dot market"/><Tip term="Mercado regional"/></span><span><i className="dot bank"/><Tip term="Banco central"/></span><span><i className="line"/><Tip term="Comercio"/></span><span><i className="line blue"/><Tip term="Flujo monetario"/></span><span><i className="ring"/><Tip term="Crecimiento"/></span></div></div>
      <aside><section className="panel"><h2><Tip term="Política monetaria"/></h2><p className="muted">Los cambios se aplican al motor en vivo, incluso durante Play.</p><div className="phase"><small><Tip term="Fase vigente"/> · <Tip term={m?.monetary_regime??'commodity'}/> · tick {activePhase?.apply_tick??0}</small><strong title={activePhase?.description}>{activePhase?.name??'Génesis monetaria'}</strong><span>{activePhase?.description??'Dotación inicial igual para todos los agentes.'}</span></div><Slider label="Tasa de interés" value={p?.base_interest_rate??.04} min={-.02} max={.2} step={.005} suffix="%" onChange={v=>updatePolicy('base_interest_rate',v)}/><Slider label="Inyección de liquidez" value={p?.liquidity_injection??0} min={0} max={50000} step={500} suffix=" u.m." onChange={v=>updatePolicy('liquidity_injection',v)}/><div className="policy-readout"><span><Tip term="Creado acumulado"/></span><strong>{m?compact(m.cumulative_money_created):'—'}</strong></div><div className="policy-readout"><span><Tip term="Deuda creada este tick"/></span><strong>{m?compact(m.debt_created_this_tick):'—'}</strong></div></section><section className="panel timeline"><h2><Tip term="Evolución del dinero"/></h2><small className="timeline-count"><Tip term="Hitos históricos"/>: {snapshot?.monetary_history.length??0}</small>{(snapshot?.monetary_history??[]).map(event=><div className={`timeline-item ${event.apply_tick<=activePolicyTick?'applied':''} ${event.catastrophe?'catastrophe':''}`} key={event.apply_tick} title={event.description}><i/><div><small><Tip term={event.era}/> · Tick {event.apply_tick}{event.optional?' · ':''}{event.optional?<Tip term="opcional"/>:''}</small><strong>{event.name}</strong><span>{event.description}</span></div></div>)}</section>{selected?<section className="panel inspector"><h2><Tip term="Nodo seleccionado"/></h2><div className="selected-id">{selected.kind==='central_bank'?'BANCO CENTRAL':selected.kind==='market'?'MERCADO':'PERSONA'} #{selected.id}</div><div className="detail"><span><Tip term="Patrimonio"/></span><strong>{selected.wealth.toFixed(0)} u.m.</strong></div><div className="detail"><span><Tip term="Variación"/></span><strong className={selected.wealth_delta>=0?'positive':'negative'}>{selected.wealth_delta>=0?'+':''}{selected.wealth_delta.toFixed(0)} u.m.</strong></div><div className="detail"><span><Tip term="Región"/></span><strong>{selected.region}</strong></div><div className="detail"><span><Tip term="Estado"/></span><strong>{selected.employed?'Activo':'Sin empleo'}</strong></div><div className="detail"><span><Tip term="Riesgo"/></span><strong>{(selected.risk_score*100).toFixed(0)}%</strong></div></section>:<section className="panel"><h2>Selecciona un nodo</h2><p className="muted">Todos comienzan con el mismo patrimonio. El tamaño se separa cuando la economía empieza a evolucionar.</p></section>}</aside></section>
    <section className="telemetry"><div><h2><Tip term="Estadísticas en tiempo real"/></h2><p className="muted">Últimos {history.length} ticks de la corrida actual.</p></div><div className="charts"><MetricChart title="PIB" values={history.map(s=>s.metrics.gdp)}/><MetricChart title="Gini" values={history.map(s=>s.metrics.gini)}/><MetricChart title="Oferta monetaria" values={history.map(s=>s.metrics.money_supply)}/><MetricChart title="Desempleo" values={history.map(s=>s.metrics.unemployment*100)}/></div></section>
  </main>
}
function compact(n:number){return n>=1000000?`${(n/1000000).toFixed(1)}M`:n>=1000?`${(n/1000).toFixed(1)}k`:n.toFixed(0)}
function Stat({label,value}:{label:string;value:string|number}){return <div className="stat"><small><Tip term={label}/></small><strong>{value}</strong></div>}
function Slider({label,value,min,max,step,suffix,onChange}:{label:string;value:number;min:number;max:number;step:number;suffix:string;onChange:(v:number)=>void}){return <label className="slider"><span><Tip term={label}/><b>{suffix==='%'?(value*100).toFixed(1)+'%':compact(value)+suffix}</b></span><input aria-label={label} title={TERM_HELP[label]} type="range" min={min} max={max} step={step} value={value} onChange={e=>onChange(Number(e.target.value))}/></label>}
function MetricChart({title,values}:{title:string;values:number[]}){const max=Math.max(...values,1),min=Math.min(...values,0);const path=values.map((v,i)=>`${(i/Math.max(values.length-1,1))*100},${94-((v-min)/Math.max(max-min,1))*78}`).join(' ');return <div className="chart"><small><Tip term={title}/></small><strong>{values.length?values[values.length-1].toFixed(title==='Gini'?3:0):'—'}</strong><svg viewBox="0 0 100 100" preserveAspectRatio="none"><polyline points={path} fill="none"/></svg></div>}

createRoot(document.getElementById('root')!).render(<App />);
