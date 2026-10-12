import { useEffect, useRef } from 'react';
import { Application, Graphics, Rectangle } from 'pixi.js';
import './network.css';

type Agent = { id:number; kind:string; region:number; x:number; y:number; size:number; risk_score:number; employed:boolean; wealth:number; wealth_delta:number };
type Edge = { source:number; target:number; kind:string; weight:number };
type Snapshot = { agents:Agent[]; edges:Edge[] };

export function NetworkCanvas({snapshot,onSelect}:{snapshot:Snapshot|null;onSelect:(agent:Agent)=>void}){
  const hostRef=useRef<HTMLDivElement>(null); const snapshotRef=useRef<Snapshot|null>(snapshot); const selectRef=useRef(onSelect); const sceneRef=useRef<{edges:Graphics;nodes:Graphics;glow:Graphics;app:Application}|null>(null);
  snapshotRef.current=snapshot; selectRef.current=onSelect;

  useEffect(()=>{
    const host=hostRef.current; if(!host) return; let cancelled=false;
    const boot=async()=>{
      const app=new Application();
      await app.init({resizeTo:host,preference:'webgl',antialias:false,backgroundAlpha:0,autoDensity:true,powerPreference:'high-performance'});
      if(cancelled){app.destroy(true);return;}
      host.appendChild(app.canvas); app.stage.eventMode='static'; app.stage.hitArea=new Rectangle(0,0,host.clientWidth,host.clientHeight);
      const edges=new Graphics(),nodes=new Graphics(),glow=new Graphics(); app.stage.addChild(edges,glow,nodes); sceneRef.current={edges,nodes,glow,app};
      app.stage.on('pointertap',(event)=>{const current=snapshotRef.current;if(!current)return;const scaleX=app.renderer.width/100,scaleY=app.renderer.height/100;const x=event.global.x/scaleX,y=event.global.y/scaleY;let closest:Agent|undefined;let distance=Infinity;for(const agent of current.agents){const dx=agent.x*100-x,dy=agent.y*100-y,d=Math.hypot(dx,dy);if(d<distance){distance=d;closest=agent;}}if(closest&&distance<Math.max(2.5,closest.size/3))selectRef.current(closest);});
      redraw(currentSnapshot(snapshotRef.current));
    };
    boot(); return()=>{cancelled=true;const scene=sceneRef.current;scene?.app.destroy(true,{children:true});sceneRef.current=null;};
  },[]);

  useEffect(()=>{redraw(snapshot)},[snapshot]);
  function currentSnapshot(value:Snapshot|null){return value;}
  function redraw(value:Snapshot|null){const scene=sceneRef.current;if(!scene||!value)return;const {app,edges,nodes,glow}=scene;const width=app.renderer.width,height=app.renderer.height;app.stage.hitArea=new Rectangle(0,0,width,height);const byId=new Map(value.agents.map(a=>[a.id,a]));edges.clear();nodes.clear();glow.clear();for(const edge of value.edges){const source=byId.get(edge.source),target=byId.get(edge.target);if(!source||!target)continue;edges.moveTo(source.x*width,source.y*height).lineTo(target.x*width,target.y*height).stroke({color:edge.kind==='issuance'?0x9c6838:edge.kind==='market'?0x587fa1:0x405b49,width:Math.max(0.5,edge.weight*.5),alpha:edge.kind==='issuance'?.42:edge.kind==='market'?.16:.22});}for(const agent of value.agents){const x=agent.x*width,y=agent.y*height,r=agent.kind==='central_bank'?12:agent.kind==='market'?9:Math.max(2,Math.min(10,agent.size*1.15));const color=agent.kind==='central_bank'?0x9c6838:agent.kind==='market'?0x244b67:agent.risk_score>.8?0xbd5b49:0x466653;if(agent.wealth_delta>0&&agent.kind==='person')glow.circle(x,y,r+3).stroke({color:0xbdd96c,width:1.5,alpha:.9});nodes.circle(x,y,r).fill({color,alpha:.86});if(agent.kind!=='person')nodes.circle(x,y,r*.45).fill({color:0xe6f1e7,alpha:.55});}}
  return <div className="network-canvas" ref={hostRef} aria-label="Mapa GPU de nodos económicos"/>;
}
