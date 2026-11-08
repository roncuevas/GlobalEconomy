import { useId, type ReactNode } from 'react';

export const API = 'http://localhost:8080';
export const TERM_HELP: Record<string,string> = {
  'PIB':'Producto Interno Bruto: valor agregado de la actividad económica simulada.',
  'u. m.':'Unidades monetarias sintéticas del laboratorio; no representan una moneda real.',
  'Manual':'El usuario decide cuándo activar la siguiente política.',
  'Automática':'El motor activa las políticas al llegar a sus ticks programados.',
  'opcional':'Evento que solo aparece cuando se activan las catástrofes opcionales.',
  'Transacción':'Movimiento económico registrado entre dos entidades del sistema.',
  'Transacciones recientes':'Movimientos institucionales registrados en el último tick.',
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
  'Integridad económica':'Reglas sintéticas y auditables que señalan flujos incompatibles con la actividad declarada.',
  'Evolución':'Permite que los agentes ajusten su estrategia según patrimonio, necesidad y resultados observados.',
  'Catástrofes':'Activa shocks sintéticos como burbujas, guerras, pandemias o crisis cambiarias.',
  'Shock externo':'Evento sintético que altera actividad, riqueza o confianza en una región y queda en el replay.',
  'Intensidad':'Magnitud del shock: proporción de actividad o riqueza que se ve afectada en la región.',
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
  'Quiebras':'Número acumulado de empresas que fueron liquidadas por insolvencia.',
  'Concentración bancaria':'Participación de los préstamos concentrada en el banco más expuesto.',
  'Brecha regional':'Diferencia relativa entre el promedio de riqueza regional más alto y más bajo.',
  'Movilidad social':'Porcentaje de personas que cambió de decil de riqueza respecto al tick anterior.',
  'Nueva corrida':'Crea una simulación nueva con la configuración actual. La corrida anterior queda guardada en Comparar escenarios.',
  'Reproducir':'Inicia la evolución automática de la corrida. Atajo: barra espaciadora.',
  'Pausar':'Detiene temporalmente la evolución y conserva el estado actual. Atajo: barra espaciadora.',
  'Atajos de teclado':'Espacio reproduce o pausa · flecha derecha avanza 1 tick · Mayús + flecha derecha avanza 10 ticks. No funcionan mientras escribes en un campo.',
  'Avanzar':'Avanza la corrida pausada. Atajo: flecha derecha avanza un tick.',
  'Vista rápida':'Filtros que reducen visualmente los nodos mostrados sin alterar la economía.',
  'Tipo':'Filtra por clase de entidad económica.',
  'Dinero':'Filtra por patrimonio relativo a la media de las personas.',
  'Solo riesgo alto':'Muestra únicamente personas cuyo riesgo individual supera el umbral alto.',
  'Modo de política':'Elige si las políticas históricas avanzan automáticamente o mediante intervención manual.',
  'Aplicar siguiente':'Activa la siguiente política monetaria disponible sin avanzar el reloj.',
  'Aplicar contramedida':'Responde a la alerta abierta y registra el efecto antes/después.',
  'Red económica':'Mapa de entidades y relaciones económicas en dos dimensiones.',
  'Persona':'Agente que recibe ingresos, consume, trabaja y acumula patrimonio.',
  'Mercado regional':'Punto de encuentro de oferta, demanda y comercio de una región.',
  'Banco central':'Entidad que modifica tasas, liquidez y reglas monetarias.',
  'Comercio':'Conexión de una persona con su mercado regional.',
  'Flujo monetario':'Conexión que representa creación o movimiento institucional de dinero.',
  'Crecimiento':'Borde verde que indica aumento de patrimonio respecto al tick anterior.',
  'Ver tick':'Vista de solo lectura de un snapshot histórico de la corrida.',
  'Intervenciones':'Acciones que modifican la corrida y quedan registradas en su replay.',
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
  'Génesis monetaria':'Inicio del experimento: todos reciben la misma dotación monetaria.',
  'Comparar escenarios':'Compara métricas de varias corridas en el mismo tick, incluso usando snapshots históricos persistidos.',
  'Tick histórico':'Tick exacto que se desea contrastar; vacío usa el último snapshot disponible.',
  'Descargar Parquet':'Descarga la serie compacta de métricas para analizarla con DuckDB, Python o notebooks.',
  'Caja':'Efectivo operativo disponible para una empresa o institución.',
  'Ventas':'Ingresos de una empresa por bienes y servicios vendidos.',
  'Deuda':'Saldo pendiente de préstamos de la entidad.',
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
  'Aplicar shock':'Inyecta un evento sintético en una región y conserva su impacto en el replay.',
  'Impuesto sobre ingresos':'Porcentaje del ingreso laboral que recauda el gobierno en cada tick.',
  'Transferencia por persona':'Pago público universal que se entrega a cada persona en cada tick.',
  'Semilla':'Número que hace reproducible la aleatoriedad de una corrida.',
  'Agentes':'Cantidad de personas creadas al iniciar la corrida.',
  'Linaje':'Familia que desciende de un mismo fundador; comparte apellido y color en el mapa.',
  'Generación':'Distancia al fundador: los fundadores son la generación 0, sus hijos la 1 y así sucesivamente.',
  'Dote':'Parte del dinero del progenitor que recibe cada hijo al nacer (25%).',
  'Herencia':'Al morir, el dinero de una persona se reparte por igual entre sus hijos vivos, después del impuesto.',
  'Impuesto a la herencia':'Proporción de cada herencia que pasa al fondo público y se reparte por igual entre toda la población.',
  'Rendimiento del capital':'Lo que gana cada año el dinero ya acumulado. Si supera al crecimiento de los salarios, las fortunas heredadas crecen más rápido que las ganadas.',
  'Riqueza heredada':'Parte de la riqueza de los vivos que llegó por dote, herencia o fondo público, frente a la que ganaron con su trabajo y sus ahorros.',
  'Población máxima':'Personas vivas que pueden sostener los recursos. Cerca del tope nacen menos hijos.',
  'Fundadores':'Familias iniciales. Todas empiezan con el mismo dinero.',
};

export async function api<T>(path:string, init?:RequestInit):Promise<T>{
  let response:Response;
  try { response=await fetch(`${API}${path}`,init); }
  catch { throw new Error('No se pudo conectar con el motor en localhost:8080. Comprueba que el servidor (cargo run) siga activo y vuelve a intentarlo.'); }
  if(!response.ok){
    if(response.status===410)throw new Error('Ese tick ya salió de la ventana local de historial. Elige un tick más reciente.');
    const detail=(await response.text().catch(()=>'')).slice(0,160);
    throw new Error(`El motor rechazó la operación (${response.status})${detail?`: ${detail}`:''}.`);
  }
  return response.json() as Promise<T>;
}
export const postJson=(body:unknown):RequestInit=>({method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(body)});

export function Tip({term, label, children}: {term:string; label?:string; children?:ReactNode}) {
  const id=useId();
  const explanation = TERM_HELP[term];
  if(!explanation)return <>{children ?? label ?? term}</>;
  return <span className="term-tip" aria-describedby={id}>{children ?? label ?? term}<span className="term-tip-bubble" id={id} role="tooltip" aria-hidden="true">{explanation}</span></span>;
}

export const isNum=(n:number|undefined|null):n is number=>typeof n==='number'&&Number.isFinite(n);
export function num(n:number|undefined|null,digits=0){return isNum(n)?n.toLocaleString('es-MX',{minimumFractionDigits:digits,maximumFractionDigits:digits}):'—'}
export function pct(n:number|undefined|null,digits=1){return isNum(n)?`${(n*100).toLocaleString('es-MX',{minimumFractionDigits:digits,maximumFractionDigits:digits})}%`:'—'}
export function compact(n:number){const abs=Math.abs(n);return abs>=1_000_000?`${(n/1_000_000).toLocaleString('es-MX',{maximumFractionDigits:1})} M`:abs>=10_000?`${(n/1000).toLocaleString('es-MX',{maximumFractionDigits:1})} k`:n.toLocaleString('es-MX',{maximumFractionDigits:0})}
export function money(n:number|undefined|null,digits=0){return isNum(n)?`${digits?n.toLocaleString('es-MX',{minimumFractionDigits:digits,maximumFractionDigits:digits}):compact(n)} u. m.`:'—'}
