import { useEffect, useRef, useState } from 'react';
import { Application, Container, Graphics, Rectangle } from 'pixi.js';
import './network.css';

type Agent = { id:number; kind:string; region:number; x:number; y:number; size:number; risk_score:number; employed:boolean; wealth:number; wealth_delta:number };
type Edge = { source:number; target:number; kind:string; weight:number };
type Snapshot = { agents:Agent[]; edges:Edge[] };
type Scene = { edges:Graphics; nodes:Graphics; glow:Graphics; world:Container; app:Application; zoom:number };

export function NetworkCanvas({snapshot,onSelect}:{snapshot:Snapshot|null;onSelect:(agent:Agent)=>void}){
  const hostRef=useRef<HTMLDivElement>(null); const snapshotRef=useRef<Snapshot|null>(snapshot); const selectRef=useRef(onSelect); const sceneRef=useRef<Scene|null>(null); const movedRef=useRef(false); const [zoom,setZoom]=useState(1);
  snapshotRef.current=snapshot; selectRef.current=onSelect;

  function setCamera(next:number){const scene=sceneRef.current;if(!scene)return;scene.zoom=Math.max(.35,Math.min(6,next));scene.world.scale.set(scene.zoom);setZoom(scene.zoom);}
  function zoomBy(factor:number){const scene=sceneRef.current;if(scene)setCamera(scene.zoom*factor);}
  function resetCamera(){const scene=sceneRef.current;if(!scene)return;scene.world.position.set(0,0);setCamera(1);}
  function fullscreen(){hostRef.current?.requestFullscreen?.();}

  useEffect(()=>{
    const host=hostRef.current; if(!host) return; let cancelled=false; let dragging=false; let lastX=0; let lastY=0;
    const boot=async()=>{
      const app=new Application();
      await app.init({resizeTo:host,preference:'webgl',antialias:false,backgroundAlpha:0,autoDensity:true,powerPreference:'high-performance'});
      if(cancelled){app.destroy(true);return;}
      host.appendChild(app.canvas); app.stage.eventMode='static'; app.stage.hitArea=new Rectangle(0,0,host.clientWidth,host.clientHeight);
      const world=new Container(), edges=new Graphics(),nodes=new Graphics(),glow=new Graphics(); world.addChild(edges,glow,nodes); app.stage.addChild(world); sceneRef.current={edges,nodes,glow,world,app,zoom:1};
      app.stage.on('pointertap',(event)=>{if(movedRef.current){movedRef.current=false;return;}const current=snapshotRef.current;if(!current)return;const scene=sceneRef.current;if(!scene)return;const width=app.renderer.width,height=app.renderer.height;const x=(event.global.x-scene.world.position.x)/scene.world.scale.x,y=(event.global.y-scene.world.position.y)/scene.world.scale.y;let closest:Agent|undefined;let distance=Infinity;for(const agent of current.agents){const dx=agent.x*width-x,dy=agent.y*height-y,d=Math.hypot(dx,dy);if(d<distance){distance=d;closest=agent;}}if(closest&&distance<Math.max(12,closest.size*1.8))selectRef.current(closest);});
      redraw(snapshotRef.current);
    };
    const onWheel=(event:WheelEvent)=>{event.preventDefault();zoomBy(event.deltaY<0?1.14:.88);};
    const onPointerDown=(event:PointerEvent)=>{if((event.target as HTMLElement).closest('button'))return;dragging=true;movedRef.current=false;lastX=event.clientX;lastY=event.clientY;host.setPointerCapture?.(event.pointerId);};
    const onPointerMove=(event:PointerEvent)=>{if(!dragging)return;const scene=sceneRef.current;if(!scene)return;const dx=event.clientX-lastX,dy=event.clientY-lastY;if(Math.abs(dx)+Math.abs(dy)>2)movedRef.current=true;scene.world.position.x+=dx;scene.world.position.y+=dy;lastX=event.clientX;lastY=event.clientY;};
    const onPointerUp=(event:PointerEvent)=>{dragging=false;host.releasePointerCapture?.(event.pointerId);};
    host.addEventListener('wheel',onWheel,{passive:false});host.addEventListener('pointerdown',onPointerDown);host.addEventListener('pointermove',onPointerMove);host.addEventListener('pointerup',onPointerUp);host.addEventListener('pointercancel',onPointerUp);
    boot(); return()=>{cancelled=true;host.removeEventListener('wheel',onWheel);host.removeEventListener('pointerdown',onPointerDown);host.removeEventListener('pointermove',onPointerMove);host.removeEventListener('pointerup',onPointerUp);host.removeEventListener('pointercancel',onPointerUp);const scene=sceneRef.current;scene?.app.destroy(true,{children:true});sceneRef.current=null;};
  },[]);

  useEffect(()=>{redraw(snapshot)},[snapshot]);
  function redraw(value:Snapshot|null){const scene=sceneRef.current;if(!scene||!value)return;const {app,edges,nodes,glow,world}=scene;const width=app.renderer.width,height=app.renderer.height;app.stage.hitArea=new Rectangle(0,0,width,height);const byId=new Map(value.agents.map(a=>[a.id,a]));edges.clear();nodes.clear();glow.clear();for(const edge of value.edges){const source=byId.get(edge.source),target=byId.get(edge.target);if(!source||!target)continue;edges.moveTo(source.x*width,source.y*height).lineTo(target.x*width,target.y*height).stroke({color:edge.kind==='issuance'?0x9c6838:edge.kind==='market'?0x587fa1:0x405b49,width:Math.max(.5,edge.weight*.5),alpha:edge.kind==='issuance'?.42:edge.kind==='market'?.16:.22});}for(const agent of value.agents){const x=agent.x*width,y=agent.y*height,r=agent.kind==='central_bank'?12:agent.kind==='market'?9:Math.max(2,Math.min(10,agent.size*1.15));const color=agent.kind==='central_bank'?0x9c6838:agent.kind==='market'?0x244b67:agent.risk_score>.8?0xbd5b49:0x466653;if(agent.wealth_delta>0&&agent.kind==='person')glow.circle(x,y,r+3).stroke({color:0xbdd96c,width:1.5,alpha:.9});nodes.circle(x,y,r).fill({color,alpha:.86});if(agent.kind!=='person')nodes.circle(x,y,r*.45).fill({color:0xe6f1e7,alpha:.55});}world.sortableChildren=false;}
  return <div className="network-canvas" ref={hostRef} aria-label="Mapa GPU de nodos económicos"><div className="network-toolbar" role="toolbar" aria-label="Controles del mapa"><button type="button" onClick={()=>zoomBy(1.25)} title="Acercar">+</button><button type="button" onClick={()=>zoomBy(.8)} title="Alejar">−</button><button type="button" onClick={resetCamera} title="Restablecer vista">100%</button><button type="button" onClick={fullscreen} title="Pantalla completa">⛶</button><span>{Math.round(zoom*100)}%</span></div><div className="network-hint">Arrastra para mover · rueda para zoom · clic en un nodo para inspeccionarlo</div></div>;
}
