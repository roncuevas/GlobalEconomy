import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { createRoot } from 'react-dom/client';
import { create } from 'zustand';
import { NetworkCanvas } from './NetworkCanvas';
import './style.css';
import './risk.css';

type Metrics = { gdp:number; gini:number; unemployment:number; inflation:number; tax_revenue:number; trade_volume:number; alerts:number; mean_wealth:number; money_supply:number; interest_rate:number; top_10_wealth_share:number; money_created_this_tick:number; cumulative_money_created:number; government_debt:number; private_debt:number; debt_created_this_tick:number; monetary_regime:string; wealth_shock:number; adverse_events:number; open_risks:number; countermeasures:number; systemic_risk:number; security_level:number; aml_alerts:number; tax_alerts:number };
type Policy = { income_tax_rate:number; transfer_per_person:number; base_interest_rate:number; liquidity_injection:number };
type MonetaryPolicy = { apply_tick:number; era:string; name:string; description:string; base_interest_rate:number; liquidity_injection:number; government_debt_change:number; private_credit_creation:number; wealth_shock:number; catastrophe:boolean; optional:boolean };
type MoneyEvent = { tick:number; kind:string; amount:number; reason:string };
type Transaction = { id:number; tick:number; from:number; to:number; kind:string; amount:number; region:number; reason:string };
type Agent = { id:number; kind:string; region:number; sector:number; x:number; y:number; size:number; risk_score:number; employed:boolean; wealth:number; wealth_delta:number };
type Edge = { source:number; target:number; kind:string; weight:number };
type MetricFrame = { tick:number; gdp:number; gini:number; unemployment:number; inflation:number; money_supply:number; government_debt:number; private_debt:number; mean_wealth:number; systemic_risk:number; security_level:number };
type RiskEvent = { id:number; policy_tick:number; policy_name:string; category:string; name:string; description:string; detection_reason:string; severity:number; detected_at:number; countermeasure_name:string; countermeasure_description:string; countermeasure_applied:boolean; before:MetricFrame; after_policy:MetricFrame|null; after_countermeasure:MetricFrame|null };
type ComplianceAlert = { id:number; tick:number; entity_id:number; category:string; severity:number; reason:string; related_transaction_ids:number[]; status:string };
type ExternalShock = { id:number; tick:number; kind:string; region:number; intensity:number; reason:string };
type Snapshot = { tick:number; agents:Agent[]; edges:Edge[]; metrics:Metrics; policy:Policy; monetary_history:MonetaryPolicy[]; money_events:MoneyEvent[]; active_policy:MonetaryPolicy|null; next_policy:MonetaryPolicy|null; policy_mode:'manual'|'automatic'; risk_events:RiskEvent[]; transactions:Transaction[]; compliance_alerts:ComplianceAlert[]; external_shocks:ExternalShock[]; companies:Company[]; banks:Bank[]; government:Government; central_bank:CentralBank; markets:Market[] };
type Company = { id:number; region:number; sector:number; cash:number; inventory:number; price:number; employees:number; production_capacity:number; sales:number; debt:number; active:boolean };
type Bank = { id:number; region:number; deposits:number; reserves:number; loans:number; capital:number; liquidity_ratio:number };
type Government = { id:number; cash:number; tax_revenue:number; spending:number; debt:number };
type CentralBank = { id:number; reserves:number; base_rate:number; liquidity_injected:number };
type Market = { id:number; region:number; sector:number; supply:number; demand:number; price:number; volume:number };
type RunSummary = { run_id:string; status:string; tick:number; created_at:string; config_hash:string; history_ticks:number[] };
type ComparisonRow = { run_id:string; tick:number; metrics:Metrics };
type MetricPoint = { tick:number; metrics:Metrics };
type Store = { snapshot:Snapshot|null; running:boolean; setSnapshot:(s:Snapshot)=>void; setRunning:(v:boolean)=>void };
const useStore = create<Store>((set)=>({snapshot:null,running:false,setSnapshot:(snapshot)=>set({snapshot}),setRunning:(running)=>set({running})}));
const API = 'http://localhost:8080';
const TERM_HELP: Record<string,string> = {
  'PIB':'Producto Interno Bruto: valor agregado de la actividad económica simulada.',
  'u. m.':'Unidades monetarias sintéticas del laboratorio; no representan una moneda real.',
  'Manual':'El usuario decide cuándo activar la siguiente política.',
  'Automática':'El motor activa las políticas al llegar a sus ticks programados.',
  'opcional':'Evento que solo aparece cuando se activan las catástrofes opcionales.',
  'Transacción':'Movimiento económico registrado entre dos entidades del sistema.',
  'Nómina':'Pago de una empresa a una persona por trabajo realizado.',
  'Impuesto':'Pago de una persona o empresa al gobierno según sus ingresos declarados.',
  'Transferencia':'Pago del gobierno a una persona sin una venta directa a cambio.',
  'Compra':'Pago de una persona a una empresa por bienes o servicios.',
  'Crédito':'Dinero creado por un banco mediante un préstamo explicable.',
  'Amortización':'Pago parcial que reduce el saldo de una deuda.',
  'Quiebra':'Liquidación de una empresa que no puede cubrir sus obligaciones.',
  'AML':'Anti-lavado: detección sintética de flujos incompatibles con la actividad declarada.',
  'Evasión fiscal':'Señal sintética de diferencia entre flujo económico, patrimonio e ingreso declarado.',
  'Alertas AML':'Casos abiertos de posible lavado detectados por reglas explicables.',
  'Alertas fiscales':'Casos abiertos de posible evasión detectados por reglas explicables.',
  'Evolución de estrategias':'Permite que los agentes ajusten su estrategia según patrimonio, necesidad y resultados observados.',
  'Shock externo':'Evento aleatorio reproducible que altera actividad, riqueza o confianza del sistema.',
  'Empresa':'Entidad que produce, contrata personas, vende, acumula caja y puede endeudarse.',
  'Banco':'Intermediario que administra depósitos, reservas y préstamos.',
  'Gobierno':'Institución que recauda impuestos, gasta, transfiere recursos y emite deuda.',
  'Mercados':'Espacios regionales donde se encuentran oferta, demanda, precios y volumen.',
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
  'Replay':'Vista de solo lectura de un snapshot histórico de la corrida.',
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
  'Comparar escenarios':'Compara métricas de varias corridas en el mismo tick, incluso usando snapshots históricos persistidos.',
  'Tick histórico':'Tick exacto que se desea contrastar; vacío usa el último snapshot disponible.',
  'Descargar Parquet':'Descarga la serie compacta de métricas para analizarla con DuckDB, Python o notebooks.',
  'Caja':'Efectivo operativo disponible para una empresa o institución.',
  'Ventas':'Ingresos de una empresa por bienes y servicios vendidos.',
  'Empleados':'Personas asignadas a una empresa en el tick actual.',
  'Depósitos':'Saldos de clientes administrados por un banco.',
  'Préstamos':'Crédito pendiente colocado por un banco.',
  'Reservas':'Liquidez que respalda depósitos y limita la expansión del crédito.',
  'Capital':'Colchón patrimonial del banco frente a pérdidas.',
  'Liquidez':'Proporción de depósitos respaldada por reservas.',
  'Oferta':'Bienes disponibles en el mercado regional.',
  'Demanda':'Necesidad de compra observada en el mercado regional.',
  'Precio':'Valor monetario sintético del bien o servicio.',
  'Volumen':'Monto intercambiado en el mercado durante el tick.',
  'Recaudación':'Impuestos recibidos por el gobierno.',
  'Gasto público':'Transferencias y gasto registrados por el gobierno.',
  'Tasa base':'Tasa de referencia fijada por el banco central.',
  'Sector':'Actividad económica o mercado productivo al que pertenece el nodo.',
  'Región filtro':'Zona geográfica sintética usada para reducir los nodos visibles.',
  'Aplicar shock':'Inyecta un evento sintético en una región y conserva su impacto en el replay.',
  'Semilla':'Número que hace reproducible la aleatoriedad de una corrida.',
  'Agentes':'Cantidad de personas creadas al iniciar la corrida.',
};

function Tip({term, children}: {term:string; children?:ReactNode}) {
  const explanation = TERM_HELP[term] ?? term;
  return <span className="term-tip" tabIndex={0} title={explanation} aria-label={`${term}: ${explanation}`}>{children ?? term}<span className="term-tip-bubble" role="tooltip">{explanation}</span></span>;
}

function nodeLabel(agent:Agent) {
  return ({person:'PERSONA',company:'EMPRESA',bank:'BANCO',market:'MERCADO',government:'GOBIERNO',central_bank:'BANCO CENTRAL'} as Record<string,string>)[agent.kind] ?? agent.kind.toUpperCase();
}

function nodeDetails(agent:Agent, snapshot:Snapshot|null) {
  if(!snapshot)return null;
  const detail=(term:string,value:ReactNode)=><div className="detail" key={term}><span><Tip term={term}/></span><strong>{value}</strong></div>;
  if(agent.kind==='company'){
    const item=snapshot.companies.find(company=>company.id===agent.id);
    return item?<>{detail('Caja',`${item.cash.toFixed(0)} u.m.`)}{detail('Ventas',`${item.sales.toFixed(0)} u.m.`)}{detail('Deuda',`${item.debt.toFixed(0)} u.m.`)}{detail('Empleados',item.employees)}{detail('Estado',item.active?'Activa':'Liquidada')}</>:null;
  }
  if(agent.kind==='bank'){
    const item=snapshot.banks.find(bank=>bank.id===agent.id);
    return item?<>{detail('Depósitos',`${item.deposits.toFixed(0)} u.m.`)}{detail('Préstamos',`${item.loans.toFixed(0)} u.m.`)}{detail('Reservas',`${item.reserves.toFixed(0)} u.m.`)}{detail('Capital',`${item.capital.toFixed(0)} u.m.`)}{detail('Liquidez',`${(item.liquidity_ratio*100).toFixed(1)}%`)}</>:null;
  }
  if(agent.kind==='market'){
    const item=snapshot.markets.find(market=>market.id===agent.id);
    return item?<>{detail('Oferta',item.supply.toFixed(0))}{detail('Demanda',item.demand.toFixed(0))}{detail('Precio',`${item.price.toFixed(2)} u.m.`)}{detail('Volumen',`${item.volume.toFixed(0)} u.m.`)}</>:null;
  }
  if(agent.kind==='government')return <>{detail('Caja',`${snapshot.government.cash.toFixed(0)} u.m.`)}{detail('Deuda pública',`${snapshot.government.debt.toFixed(0)} u.m.`)}{detail('Recaudación',`${snapshot.government.tax_revenue.toFixed(0)} u.m.`)}{detail('Gasto público',`${snapshot.government.spending.toFixed(0)} u.m.`)}</>;
  if(agent.kind==='central_bank')return <>{detail('Reservas',`${snapshot.central_bank.reserves.toFixed(0)} u.m.`)}{detail('Tasa base',`${(snapshot.central_bank.base_rate*100).toFixed(2)}%`)}{detail('Inyección de liquidez',`${snapshot.central_bank.liquidity_injected.toFixed(0)} u.m.`)}</>;
  return null;
}

function App(){
  const {snapshot,running,setSnapshot,setRunning}=useStore();
  const [id,setId]=useState(''); const [history,setHistory]=useState<Snapshot[]>([]); const [metricHistory,setMetricHistory]=useState<MetricPoint[]>([]); const [availableTicks,setAvailableTicks]=useState<number[]>([]); const [replayTick,setReplayTick]=useState<number|null>(null); const [selected,setSelected]=useState<Agent|null>(null); const [catastrophes,setCatastrophes]=useState(true); const [evolution,setEvolution]=useState(true); const [seed,setSeed]=useState(42); const [agentCount,setAgentCount]=useState('1000'); const [shockKind,setShockKind]=useState('shock_oferta'); const [shockRegion,setShockRegion]=useState(0); const [shockIntensity,setShockIntensity]=useState(.2); const [kindFilter,setKindFilter]=useState('all'); const [regionFilter,setRegionFilter]=useState('all'); const [sectorFilter,setSectorFilter]=useState('all'); const [wealthFilter,setWealthFilter]=useState('all'); const [riskOnly,setRiskOnly]=useState(false); const [runs,setRuns]=useState<RunSummary[]>([]); const [compareIds,setCompareIds]=useState<string[]>([]); const [compareTick,setCompareTick]=useState(''); const [comparison,setComparison]=useState<ComparisonRow[]>([]); const socket=useRef<WebSocket|null>(null);
  const record=(s:Snapshot)=>{setSnapshot(s);setReplayTick(null);setHistory(h=>[...h.slice(-39),s]);setMetricHistory(h=>[...h.slice(-239),{tick:s.tick,metrics:s.metrics}]);setAvailableTicks(ticks=>ticks.includes(s.tick)?ticks:[...ticks,s.tick]);};
  async function refreshRuns(){ const r=await fetch(`${API}/api/runs`); if(r.ok)setRuns(await r.json()); }
  async function createRun(){ const r=await fetch(`${API}/api/runs`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({seed,model_version:'0.4.0',agent_count:Number(agentCount),regions:8,ticks:1000,historical_mode:true,catastrophes,policy_mode:'manual',policy_config:{income_tax_rate:.18,transfer_per_person:20,base_interest_rate:.04,liquidity_injection:0},scenario_config:{food_price:12,wage:100,initial_wealth:2500,evolution,shock_probability:.02,shock_intensity:.18,strategy_mutation_rate:.02}})}); const data=await r.json(); setId(data.run_id);setReplayTick(null); const s=await fetch(`${API}/api/runs/${data.run_id}/snapshot`); const snap=await s.json(); setHistory([snap]);setSnapshot(snap); const metricResponse=await fetch(`${API}/api/runs/${data.run_id}/metrics`); if(metricResponse.ok){const points:MetricPoint[]=await metricResponse.json();setMetricHistory(points);setAvailableTicks(points.map(point=>point.tick));} await refreshRuns(); }
  async function step(){ if(!id) return; const r=await fetch(`${API}/api/runs/${id}/step`,{method:'POST'}); record(await r.json()); }
  async function updatePolicy(key:keyof Policy,value:number){ if(!id||!snapshot) return; const policy={...snapshot.policy,[key]:value}; const r=await fetch(`${API}/api/runs/${id}/policy`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(policy)}); record(await r.json()); }
  async function applyNextPolicy(){ if(!id||running) return; const r=await fetch(`${API}/api/runs/${id}/policy/next`,{method:'POST'}); if(r.ok)record(await r.json()); }
  async function updatePolicyMode(mode:'manual'|'automatic'){ if(!id) return; const r=await fetch(`${API}/api/runs/${id}/policy-mode`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({mode})}); if(r.ok)record(await r.json()); }
  async function applyCountermeasure(){ if(!id) return; const r=await fetch(`${API}/api/runs/${id}/risk/countermeasure`,{method:'POST'}); if(r.ok)record(await r.json()); }
  async function injectShock(){ if(!id||replayTick!==null) return; const r=await fetch(`${API}/api/runs/${id}/shock`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({kind:shockKind,region:shockRegion,intensity:shockIntensity})}); if(r.ok)record(await r.json()); }
  async function resolveCompliance(alertId:number){ if(!id||replayTick!==null) return; const r=await fetch(`${API}/api/runs/${id}/compliance/${alertId}/resolve`,{method:'POST'}); if(r.ok)record(await r.json()); }
  function toggle(){ if(!id) return; const next=!running; fetch(`${API}/api/runs/${id}/${next?'start':'pause'}`,{method:'POST'}); setRunning(next); if(next){ socket.current?.close(); socket.current=new WebSocket(`${API.replace('http','ws')}/api/runs/${id}/stream`); socket.current.onmessage=(e:MessageEvent<string>)=>{const m=JSON.parse(e.data);if(m.type==='snapshot'&&m.data)record(m.data);else if(m.type==='run_status'&&m.data)setRunning(m.data.status==='running');}; } else socket.current?.close(); }
  async function enterReplay(tick:number){if(running)return;const frame=history.find(item=>item.tick===tick);if(frame){setReplayTick(tick);setSnapshot(frame);setSelected(null);return;}if(!id)return;const response=await fetch(`${API}/api/runs/${id}/replay/${tick}`);if(response.ok){const persisted:Snapshot=await response.json();setReplayTick(tick);setSnapshot(persisted);setSelected(null);}}
  function exitReplay(){const latest=history[history.length-1];if(latest){setReplayTick(null);setSnapshot(latest);}}
  useEffect(()=>()=>socket.current?.close(),[]);
  useEffect(()=>{refreshRuns();},[]);
  async function compareScenarios(){ if(compareIds.length<2)return; const response=await fetch(`${API}/api/runs/compare`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({run_ids:compareIds,tick:compareTick===''?null:Number(compareTick)})}); if(response.ok){const data=await response.json();setComparison(data.runs);}}
  async function downloadParquet(){ if(!id)return; const response=await fetch(`${API}/api/runs/${id}/metrics.parquet`); if(!response.ok)return; const blob=await response.blob(); const url=URL.createObjectURL(blob); const link=document.createElement('a'); link.href=url; link.download=`run-${id}-metrics.parquet`; link.click(); URL.revokeObjectURL(url); }
  const filteredSnapshot=useMemo(()=>{if(!snapshot)return null;const people=snapshot.agents.filter(a=>a.kind==='person');const mean=people.reduce((sum,a)=>sum+a.wealth,0)/Math.max(people.length,1);const agents=snapshot.agents.filter(a=>{if(kindFilter!=='all'&&a.kind!==kindFilter)return false;if(regionFilter!=='all'&&a.region!==Number(regionFilter))return false;if(sectorFilter!=='all'&&a.sector!==Number(sectorFilter))return false;if(wealthFilter==='low'&&a.wealth>mean*.8)return false;if(wealthFilter==='middle'&&(a.wealth<=mean*.8||a.wealth>=mean*1.3))return false;if(wealthFilter==='high'&&a.wealth<mean*1.3)return false;if(riskOnly&&a.risk_score<.8)return false;return true;});const ids=new Set(agents.map(a=>a.id));return {...snapshot,agents,edges:snapshot.edges.filter(e=>ids.has(e.source)&&ids.has(e.target)).slice(0,2500)};},[snapshot,kindFilter,regionFilter,sectorFilter,wealthFilter,riskOnly]);
  const m=snapshot?.metrics; const p=snapshot?.policy; const nodes=filteredSnapshot?.agents??[]; const visibleEdges=filteredSnapshot?.edges??[]; const regionOptions=useMemo(()=>snapshot?[...new Set(snapshot.agents.map(agent=>agent.region))].sort((a,b)=>a-b):[],[snapshot]); const sectorOptions=useMemo(()=>snapshot?[...new Set(snapshot.agents.map(agent=>agent.sector))].sort((a,b)=>a-b):[],[snapshot]);
  const activePhase=snapshot?.active_policy; const nextPhase=snapshot?.next_policy; const activePolicyTick=activePhase?.apply_tick??0; const openRisk=[...(snapshot?.risk_events??[])].reverse().find(event=>!event.countermeasure_applied); const latestRisk=snapshot?.risk_events?.[snapshot.risk_events.length-1]; const riskForPanel=openRisk??latestRisk;
  return <main><header><div><span className="eyebrow">GLOBAL ECONOMY / NETWORK LAB</span><h1>Economía viva</h1><p>Mapa bidimensional de nodos interconectados. La riqueza cambia el tamaño; las relaciones muestran comercio y mercados.</p></div><div className="controls"><label className="run-param" title={TERM_HELP.Semilla}><Tip term="Semilla"/><input type="number" value={seed} onChange={e=>setSeed(Number(e.target.value))}/></label><label className="run-param" title={TERM_HELP.Agentes}><Tip term="Agentes"/><select value={agentCount} onChange={e=>setAgentCount(e.target.value)}><option value="1000">1,000</option><option value="10000">10,000</option><option value="100000">100,000</option></select></label><label className="toggle" title={TERM_HELP['Catástrofes opcionales']}><input type="checkbox" checked={catastrophes} onChange={e=>setCatastrophes(e.target.checked)}/><span><Tip term="Catástrofes opcionales"/></span></label><label className="toggle" title={TERM_HELP['Evolución de estrategias']}><input type="checkbox" checked={evolution} onChange={e=>setEvolution(e.target.checked)}/><span><Tip term="Evolución de estrategias"/></span></label><button title={TERM_HELP['Nueva corrida']} onClick={createRun}>Nueva corrida</button><button title={TERM_HELP[running?'Pausar':'Play']} disabled={!id||replayTick!==null} onClick={toggle}>{running?'Pausar':'Play'}</button><button title={TERM_HELP.Step} disabled={!id||running||replayTick!==null} onClick={step}>Step</button></div></header>
    <section className="stats"><Stat label="Tick" value={snapshot?.tick??'—'}/><Stat label="PIB simulado" value={m?m.gdp.toFixed(0):'—'}/><Stat label="Oferta monetaria" value={m?compact(m.money_supply):'—'}/><Stat label="Creado este tick" value={m?compact(m.money_created_this_tick):'—'}/><Stat label="Deuda pública" value={m?compact(m.government_debt):'—'}/><Stat label="Deuda privada" value={m?compact(m.private_debt):'—'}/><Stat label="Gini" value={m?m.gini.toFixed(3):'—'}/><Stat label="Top 10% riqueza" value={m?`${(m.top_10_wealth_share*100).toFixed(1)}%`:'—'}/></section>
    <section className="filter-strip"><strong><Tip term="Vista rápida"/></strong><label title={TERM_HELP.Tipo}>Tipo<select value={kindFilter} onChange={e=>setKindFilter(e.target.value)}><option value="all">Todos los nodos</option><option value="person">Solo personas</option><option value="company">Solo empresas</option><option value="market">Solo mercados</option><option value="bank">Solo bancos</option><option value="government">Solo gobierno</option><option value="central_bank">Solo banco central</option></select></label><label title={TERM_HELP['Región filtro']}><Tip term="Región filtro"/><select value={regionFilter} onChange={e=>setRegionFilter(e.target.value)}><option value="all">Todas</option>{regionOptions.map(region=><option key={region} value={region}>Región {region}</option>)}</select></label><label title={TERM_HELP.Sector}><Tip term="Sector"/><select value={sectorFilter} onChange={e=>setSectorFilter(e.target.value)}><option value="all">Todos</option>{sectorOptions.map(sector=><option key={sector} value={sector}>{sector===0?'General':`Sector ${sector}`}</option>)}</select></label><label title={TERM_HELP.Dinero}>Dinero<select value={wealthFilter} onChange={e=>setWealthFilter(e.target.value)}><option value="all">Cualquier patrimonio</option><option value="low">Bajo</option><option value="middle">Medio</option><option value="high">Alto</option></select></label><label className="toggle" title={TERM_HELP['Solo riesgo alto']}><input type="checkbox" checked={riskOnly} onChange={e=>setRiskOnly(e.target.checked)}/><span>Solo riesgo alto</span></label><label title={TERM_HELP.Política}>Política<select value={snapshot?.policy_mode??'manual'} onChange={e=>updatePolicyMode(e.target.value as 'manual'|'automatic')}><option value="manual">Manual</option><option value="automatic">Automática</option></select></label><button className="policy-next" title={TERM_HELP['Aplicar siguiente']} disabled={!id||running||replayTick!==null||!nextPhase} onClick={applyNextPolicy}>Aplicar siguiente{nextPhase?` · ${nextPhase.name}`:''}</button><button className="countermeasure" title={TERM_HELP['Aplicar contramedida']} disabled={!id||!openRisk||replayTick!==null} onClick={applyCountermeasure}>Aplicar contramedida{openRisk?` · ${openRisk.countermeasure_name}`:''}</button><small title="Cantidad de nodos y conexiones que cumplen los filtros actuales">{nodes.length} nodos visibles · {visibleEdges.length} conexiones</small></section>
    {riskForPanel?<section className="risk-panel"><div><small title={TERM_HELP[openRisk?'Alerta detectada':'Alerta resuelta']}>{openRisk?'ALERTA DETECTADA':'ALERTA RESUELTA'} · tick {riskForPanel.detected_at} · severidad {(riskForPanel.severity*100).toFixed(0)}%</small><h2 title={riskForPanel.description}>{riskForPanel.name}</h2><p>{riskForPanel.description}</p><p className="muted"><Tip term="Detección"/>: {riskForPanel.detection_reason}</p><p className="muted"><Tip term="Contramedida propuesta"/>: {riskForPanel.countermeasure_description}</p></div><div className="risk-stat"><span><Tip term="Riesgo sistémico"/></span><strong>{((m?.systemic_risk??0)*100).toFixed(0)}%</strong><span><Tip term="Seguridad monetaria"/></span><strong>{((m?.security_level??0)*100).toFixed(0)}%</strong></div><details open={!openRisk}><summary><Tip term="Historial de alertas"/> ({snapshot?.risk_events.length??0})</summary>{(snapshot?.risk_events??[]).slice().reverse().map(event=><div className="risk-row" key={event.id}><strong title={event.description}>{event.name}</strong><span title={event.countermeasure_applied?TERM_HELP['Contramedida aplicada']:TERM_HELP.Abierta}>tick {event.detected_at} · {event.countermeasure_applied?'contramedida aplicada':'abierta'}</span><small>Antes: riesgo {(event.before.systemic_risk*100).toFixed(0)}% · Después: {event.after_countermeasure?`riesgo ${(event.after_countermeasure.systemic_risk*100).toFixed(0)}%`:'pendiente'}</small></div>)}</details></section>:<section className="risk-panel quiet"><span>Sin riesgos monetarios abiertos detectados.</span><small>{m?.countermeasures??0} contramedidas aplicadas · {m?.adverse_events??0} eventos registrados</small></section>}
    <section className="workspace"><div className="world"><div className="map-title"><span><Tip term="Red económica 2D"/> · <Tip term="GPU"/></span><small title="Cantidad de personas, nodos y conexiones que cumplen los filtros actuales">{nodes.filter(n=>n.kind==='person').length} personas · {nodes.length} nodos · {visibleEdges.length} conexiones</small></div><NetworkCanvas snapshot={filteredSnapshot} onSelect={setSelected}/><div className="legend"><span><i className="dot person"/><Tip term="Persona"/></span><span><i className="dot" style={{background:'#c47f3a'}}/><Tip term="Empresa"/></span><span><i className="dot market"/><Tip term="Mercado regional"/></span><span><i className="dot" style={{background:'#3f6f8f'}}/><Tip term="Banco"/></span><span><i className="dot" style={{background:'#7c4d8d'}}/><Tip term="Gobierno"/></span><span><i className="dot bank"/><Tip term="Banco central"/></span><span><i className="line"/><Tip term="Comercio"/></span><span><i className="line blue"/><Tip term="Flujo monetario"/></span><span><i className="ring"/><Tip term="Crecimiento"/></span></div></div>
      <aside><section className="panel"><h2><Tip term="Política monetaria"/></h2><p className="muted">Los cambios se aplican al motor en vivo, incluso durante Play.</p><div className="phase"><small><Tip term="Fase vigente"/> · <Tip term={m?.monetary_regime??'commodity'}/> · tick {activePhase?.apply_tick??0}</small><strong title={activePhase?.description}>{activePhase?.name??'Génesis monetaria'}</strong><span>{activePhase?.description??'Dotación inicial igual para todos los agentes.'}</span></div><Slider label="Tasa de interés" value={p?.base_interest_rate??.04} min={-.02} max={.2} step={.005} suffix="%" onChange={v=>updatePolicy('base_interest_rate',v)}/><Slider label="Inyección de liquidez" value={p?.liquidity_injection??0} min={0} max={50000} step={500} suffix=" u.m." onChange={v=>updatePolicy('liquidity_injection',v)}/><div className="policy-readout"><span><Tip term="Creado acumulado"/></span><strong>{m?compact(m.cumulative_money_created):'—'}</strong></div><div className="policy-readout"><span><Tip term="Deuda creada este tick"/></span><strong>{m?compact(m.debt_created_this_tick):'—'}</strong></div></section><section className="panel timeline"><h2><Tip term="Evolución del dinero"/></h2><small className="timeline-count"><Tip term="Hitos históricos"/>: {snapshot?.monetary_history.length??0}</small>{(snapshot?.monetary_history??[]).map(event=><div className={`timeline-item ${event.apply_tick<=activePolicyTick?'applied':''} ${event.catastrophe?'catastrophe':''}`} key={event.apply_tick} title={event.description}><i/><div><small><Tip term={event.era}/> · Tick {event.apply_tick}{event.optional?' · ':''}{event.optional?<Tip term="opcional"/>:''}</small><strong>{event.name}</strong><span>{event.description}</span></div></div>)}</section>{selected?<section className="panel inspector"><h2><Tip term="Nodo seleccionado"/></h2><div className="selected-id">{nodeLabel(selected)} #{selected.id}</div><div className="detail"><span><Tip term="Patrimonio"/></span><strong>{selected.wealth.toFixed(0)} u.m.</strong></div><div className="detail"><span><Tip term="Variación"/></span><strong className={selected.wealth_delta>=0?'positive':'negative'}>{selected.wealth_delta>=0?'+':''}{selected.wealth_delta.toFixed(0)} u.m.</strong></div><div className="detail"><span><Tip term="Región"/></span><strong>{selected.region}</strong></div><div className="detail"><span><Tip term="Estado"/></span><strong>{selected.employed?'Activo':'Sin empleo'}</strong></div><div className="detail"><span><Tip term="Riesgo"/></span><strong>{(selected.risk_score*100).toFixed(0)}%</strong></div>{nodeDetails(selected,snapshot)}</section>:<section className="panel"><h2>Selecciona un nodo</h2><p className="muted">Todos comienzan con el mismo patrimonio. El tamaño se separa cuando la economía empieza a evolucionar.</p></section>}</aside></section>
    <section className="telemetry"><div><h2><Tip term="Estadísticas en tiempo real"/></h2><p className="muted">Últimos {metricHistory.length} ticks persistidos de la corrida actual.</p><button type="button" title={TERM_HELP['Descargar Parquet']} disabled={!id||metricHistory.length===0} onClick={downloadParquet}><Tip term="Descargar Parquet"/></button>{availableTicks.length>1&&<label className="replay-control" title={TERM_HELP.Replay}><Tip term="Replay"/><select value={replayTick??''} onChange={e=>e.target.value===''?exitReplay():void enterReplay(Number(e.target.value))}><option value="">Estado vivo</option>{availableTicks.map(tick=><option key={tick} value={tick}>Tick {tick}</option>)}</select></label>}{replayTick!==null&&<button type="button" onClick={exitReplay}>Volver al estado vivo</button>}</div><div className="charts"><MetricChart title="PIB" values={metricHistory.map(s=>s.metrics.gdp)}/><MetricChart title="Gini" values={metricHistory.map(s=>s.metrics.gini)}/><MetricChart title="Oferta monetaria" values={metricHistory.map(s=>s.metrics.money_supply)}/><MetricChart title="Desempleo" values={metricHistory.map(s=>s.metrics.unemployment*100)}/></div></section>
    <section className="panel comparison-panel"><div className="comparison-head"><div><h2>Comparar escenarios</h2><p className="muted">Selecciona dos o más corridas para medir el efecto de sus políticas y shocks.</p></div><button type="button" disabled={compareIds.length<2} onClick={compareScenarios}>Comparar</button></div><div className="run-picker">{runs.slice().reverse().slice(0,8).map(run=><label key={run.run_id} title={`Tick actual ${run.tick} · hash ${run.config_hash}`}><input type="checkbox" checked={compareIds.includes(run.run_id)} onChange={e=>setCompareIds(ids=>e.target.checked?[...ids,run.run_id]:ids.filter(item=>item!==run.run_id))}/><span>{run.run_id.slice(0,8)} · tick {run.tick}</span></label>)}</div><label className="compare-tick">Tick histórico <input type="number" min="0" placeholder="último disponible" value={compareTick} onChange={e=>setCompareTick(e.target.value)}/></label>{comparison.length>0&&<div className="comparison-table"><div className="comparison-row comparison-title"><span>Corrida</span><span>Tick</span><span>PIB</span><span>Gini</span><span>Dinero</span><span>Desempleo</span></div>{comparison.map(row=><div className="comparison-row" key={row.run_id}><strong>{row.run_id.slice(0,8)}</strong><span>{row.tick}</span><span>{compact(row.metrics.gdp)}</span><span>{row.metrics.gini.toFixed(3)}</span><span>{compact(row.metrics.money_supply)}</span><span>{(row.metrics.unemployment*100).toFixed(1)}%</span></div>)}</div>}</section>
    <section className="panel transactions-panel"><h2><Tip term="Transacción"/> recientes</h2><p className="muted"><Tip term="Transacción"/> institucionales del último tick, agrupadas por tipo y región.</p><div className="transaction-list">{(snapshot?.transactions??[]).slice().reverse().slice(0,12).map(tx=><div className="transaction-row" key={tx.id}><strong>{transactionLabel(tx.kind)}</strong><span>{entityLabel(tx.from)} → {entityLabel(tx.to)}</span><b>{compact(tx.amount)} u.m.</b><small>tick {tx.tick} · región {tx.region}</small></div>)}</div></section>
    <section className="panel compliance-panel"><h2>Integridad económica</h2><div className="compliance-summary"><span><Tip term="Alertas AML"/><strong>{m?.aml_alerts??0}</strong></span><span><Tip term="Alertas fiscales"/><strong>{m?.tax_alerts??0}</strong></span></div><p className="muted">Reglas sintéticas, defensivas y auditables; cada caso conserva la razón y las transacciones relacionadas.</p><div className="compliance-list">{(snapshot?.compliance_alerts??[]).slice().reverse().slice(0,8).map(alert=><div className="compliance-row" key={alert.id}><strong>{alert.category==='aml'?'AML':'Evasión fiscal'} · {entityLabel(alert.entity_id)}</strong><span>severidad {(alert.severity*100).toFixed(0)}% · tick {alert.tick} · {alert.status==='open'?'abierta':'resuelta'}</span><small>{alert.reason}</small>{alert.status==='open'&&<button type="button" onClick={()=>resolveCompliance(alert.id)} disabled={replayTick!==null}>Marcar como revisada</button>}</div>)}</div></section>
    <section className="panel shock-panel"><h2><Tip term="Shock externo"/></h2><p className="muted">Eventos aleatorios reproducibles activados por la configuración de catástrofes.</p><div className="shock-controls"><label>Tipo<select value={shockKind} onChange={e=>setShockKind(e.target.value)}><option value="shock_oferta">Shock de oferta</option><option value="corrida_bancaria">Corrida bancaria</option><option value="burbuja_activos">Burbuja de activos</option><option value="crisis_sanitaria">Crisis sanitaria</option></select></label><label>Región<select value={shockRegion} onChange={e=>setShockRegion(Number(e.target.value))}>{regionOptions.map(region=><option key={region} value={region}>Región {region}</option>)}</select></label><label>Intensidad<input type="range" min="0.05" max="0.8" step="0.05" value={shockIntensity} onChange={e=>setShockIntensity(Number(e.target.value))}/><span>{(shockIntensity*100).toFixed(0)}%</span></label><button type="button" title={TERM_HELP['Aplicar shock']} disabled={!id||replayTick!==null} onClick={injectShock}><Tip term="Aplicar shock"/></button></div><div className="compliance-list">{(snapshot?.external_shocks??[]).slice().reverse().slice(0,6).map(shock=><div className="compliance-row" key={shock.id}><strong>{shock.kind} · región {shock.region}</strong><span>tick {shock.tick} · impacto {(shock.intensity*100).toFixed(1)}%</span><small>{shock.reason}</small></div>)}</div></section>
  </main>
}
function compact(n:number){return n>=1000000?`${(n/1000000).toFixed(1)}M`:n>=1000?`${(n/1000).toFixed(1)}k`:n.toFixed(0)}
function transactionLabel(kind:string){return ({wage:'Nómina',tax:'Impuesto',transfer:'Transferencia',purchase:'Compra',loan:'Crédito',repayment:'Amortización',bankruptcy:'Quiebra'} as Record<string,string>)[kind]??kind}
function entityLabel(id:number){if(id>=6_000_000)return 'Banco central';if(id>=5_000_000)return 'Gobierno';if(id>=4_000_000)return 'Banco';if(id>=3_000_000)return 'Empresa';return `Persona #${id}`}
function Stat({label,value}:{label:string;value:string|number}){return <div className="stat"><small><Tip term={label}/></small><strong>{value}</strong></div>}
function Slider({label,value,min,max,step,suffix,onChange}:{label:string;value:number;min:number;max:number;step:number;suffix:string;onChange:(v:number)=>void}){return <label className="slider"><span><Tip term={label}/><b>{suffix==='%'?(value*100).toFixed(1)+'%':compact(value)+suffix}</b></span><input aria-label={label} title={TERM_HELP[label]} type="range" min={min} max={max} step={step} value={value} onChange={e=>onChange(Number(e.target.value))}/></label>}
function MetricChart({title,values}:{title:string;values:number[]}){const max=Math.max(...values,1),min=Math.min(...values,0);const path=values.map((v,i)=>`${(i/Math.max(values.length-1,1))*100},${94-((v-min)/Math.max(max-min,1))*78}`).join(' ');return <div className="chart"><small><Tip term={title}/></small><strong>{values.length?values[values.length-1].toFixed(title==='Gini'?3:0):'—'}</strong><svg viewBox="0 0 100 100" preserveAspectRatio="none"><polyline points={path} fill="none"/></svg></div>}

createRoot(document.getElementById('root')!).render(<App />);
