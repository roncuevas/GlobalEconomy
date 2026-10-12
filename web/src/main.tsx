import { useEffect, useRef, useState } from 'react';
import { create } from 'zustand';
import './style.css';

type Metrics = { gdp:number; gini:number; unemployment:number; inflation:number; tax_revenue:number; trade_volume:number; alerts:number; mean_wealth:number };
type Agent = { id:number; kind:string; region:number; x:number; y:number; size:number; risk_score:number; employed:boolean };
type Snapshot = { tick:number; agents:Agent[]; metrics:Metrics };
type Store = { snapshot:Snapshot|null; running:boolean; setSnapshot:(s:Snapshot)=>void; setRunning:(v:boolean)=>void };
const useStore = create<Store>((set)=>({snapshot:null,running:false,setSnapshot:(snapshot)=>set({snapshot}),setRunning:(running)=>set({running})}));
const API = 'http://localhost:8080';

function App(){
  const {snapshot,running,setSnapshot,setRunning}=useStore(); const [id,setId]=useState(''); const socket=useRef<WebSocket|null>(null);
  async function createRun(){ const r=await fetch(`${API}/api/runs`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({seed:42,model_version:'0.1.0',agent_count:1000,regions:8,ticks:1000,policy_config:{income_tax_rate:.18,transfer_per_person:20,base_interest_rate:.04},scenario_config:{food_price:12,wage:100}})}); const data=await r.json(); setId(data.run_id); const s=await fetch(`${API}/api/runs/${data.run_id}/snapshot`); setSnapshot(await s.json()); }
  async function step(){ if(!id) return; const r=await fetch(`${API}/api/runs/${id}/step`,{method:'POST'}); setSnapshot(await r.json()); }
  function toggle(){ if(!id) return; const next=!running; fetch(`${API}/api/runs/${id}/${next?'start':'pause'}`,{method:'POST'}); setRunning(next); if(next){ socket.current?.close(); socket.current=new WebSocket(`${API.replace('http','ws')}/api/runs/${id}/stream`); socket.current.onmessage=(e:MessageEvent<string>)=>{const m=JSON.parse(e.data);setSnapshot(m.data);}; } else socket.current?.close(); }
  useEffect(()=>()=>socket.current?.close(),[]);
  const m=snapshot?.metrics;
  return <main><header><div><span className="eyebrow">GLOBAL ECONOMY / LABORATORIO</span><h1>Economía viva</h1><p>Simulación determinista de agentes, mercados y decisiones cotidianas.</p></div><div className="controls"><button onClick={createRun}>Nueva corrida</button><button disabled={!id} onClick={toggle}>{running?'Pausar':'Play'}</button><button disabled={!id||running} onClick={step}>Step</button></div></header>
    <section className="stats"><Stat label="Tick" value={snapshot?.tick??'—'}/><Stat label="PIB simulado" value={m?m.gdp.toFixed(0):'—'}/><Stat label="Gini" value={m?m.gini.toFixed(3):'—'}/><Stat label="Desempleo" value={m?`${(m.unemployment*100).toFixed(1)}%`:'—'}/><Stat label="Alertas" value={m?.alerts??'—'}/></section>
    <section className="workspace"><div className="world"><div className="map-title">Mapa económico <small>{snapshot?.agents.length??0} agentes visibles</small></div><div className="map">{snapshot?.agents.slice(0,1000).map(a=><span key={a.id} title={`Persona ${a.id} · Región ${a.region}`} className={`agent ${a.employed?'employed':''} ${a.risk_score>.8?'risky':''}`} style={{left:`${a.x*94+3}%`,top:`${a.y*90+5}%`,width:`${Math.max(4,a.size*2)}px`,height:`${Math.max(4,a.size*2)}px`}}/> )}</div></div><aside><h2>Lectura de la corrida</h2><p>La economía avanza por semanas. Cada persona recibe salario, cubre necesidades, paga impuestos y puede cambiar de empleo con aleatoriedad reproducible.</p><div className="note"><strong>Semilla</strong><span>42 · modelo 0.1.0</span></div><div className="note"><strong>Comercio</strong><span>{m?m.trade_volume.toFixed(0):'—'} unidades monetarias</span></div><div className="note"><strong>Recaudación</strong><span>{m?m.tax_revenue.toFixed(0):'—'} unidades monetarias</span></div></aside></section>
  </main>
}
function Stat({label,value}:{label:string;value:string|number}){return <div className="stat"><small>{label}</small><strong>{value}</strong></div>}
export default App;
