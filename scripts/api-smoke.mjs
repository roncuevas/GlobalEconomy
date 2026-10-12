const baseUrl = process.env.GLOBALECONOMY_BASE_URL ?? 'http://127.0.0.1:8080';

async function request(path, options = {}) {
  const response = await fetch(`${baseUrl}${path}`, options);
  const contentType = response.headers.get('content-type') ?? '';
  const body = contentType.includes('json')
    ? await response.json()
    : contentType.startsWith('text/')
      ? await response.text()
      : await response.arrayBuffer();
  if (!response.ok) {
    throw new Error(`${options.method ?? 'GET'} ${path}: HTTP ${response.status}`);
  }
  return body;
}

function json(value) {
  return {
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(value),
  };
}

const config = (seed, transfer) => ({
  seed,
  model_version: '0.4.0',
  agent_count: 100,
  regions: 4,
  ticks: 8,
  historical_mode: true,
  catastrophes: false,
  policy_mode: 'manual',
  policy_config: {
    income_tax_rate: transfer === 0 ? 0.18 : 0.45,
    transfer_per_person: transfer,
    base_interest_rate: 0.04,
    liquidity_injection: 0,
  },
  scenario_config: {
    food_price: 12,
    wage: 100,
    initial_wealth: 2500,
    evolution: false,
    shock_probability: 0,
    shock_intensity: 0.18,
    strategy_mutation_rate: 0.02,
  },
});

const health = await request('/health');
if (health !== 'ok') throw new Error('health endpoint did not return ok');

const first = await request('/api/runs', { method: 'POST', ...json(config(301, 0)) });
const second = await request('/api/runs', { method: 'POST', ...json(config(302, 120)) });

for (const run of [first, second]) {
  if (!run.run_id || run.status !== 'paused') throw new Error('run creation contract failed');
  const stepped = await request(`/api/runs/${run.run_id}/step`, { method: 'POST' });
  if (stepped.tick !== 1 || stepped.agents.length === 0) throw new Error('step contract failed');
}

const policy = await request(`/api/runs/${first.run_id}/policy`, {
  method: 'POST',
  ...json({ income_tax_rate: 0.22, transfer_per_person: 30, base_interest_rate: 0.06, liquidity_injection: 500 }),
});
if (policy.policy.base_interest_rate !== 0.06) throw new Error('policy update failed');

const shock = await request(`/api/runs/${first.run_id}/shock`, {
  method: 'POST',
  ...json({ kind: 'shock_oferta', region: 0, intensity: 0.2 }),
});
if (shock.external_shocks.length !== 1) throw new Error('shock was not persisted');

const events = await request(`/api/runs/${first.run_id}/events?type=purchase&min_amount=1`);
const transactions = await request(`/api/runs/${first.run_id}/transactions?kind=purchase&min_amount=1`);
if (!Array.isArray(events) || !Array.isArray(transactions) || transactions.length === 0) {
  throw new Error('auditable filters failed');
}

const comparison = await request('/api/runs/compare', {
  method: 'POST',
  ...json({ run_ids: [first.run_id, second.run_id], tick: 1 }),
});
if (comparison.runs.length !== 2 || comparison.runs.some((run) => run.tick !== 1)) {
  throw new Error('scenario comparison failed');
}
if (comparison.runs[0].metrics.tax_revenue === comparison.runs[1].metrics.tax_revenue) {
  throw new Error('comparison did not expose policy effect');
}

const replay = await request(`/api/runs/${first.run_id}/replay/1`);
if (replay.tick !== 1) throw new Error('replay lookup failed');
const metrics = await request(`/api/runs/${first.run_id}/metrics`);
if (metrics.length < 2) throw new Error('metric history failed');
const parquet = await request(`/api/runs/${first.run_id}/metrics.parquet`);
const magic = new TextDecoder().decode(new Uint8Array(parquet).slice(0, 4));
if (magic !== 'PAR1') throw new Error('Parquet export failed');

await request(`/api/runs/${first.run_id}/start`, { method: 'POST' });
const streamTypes = await new Promise((resolve, reject) => {
  const types = new Set();
  const socket = new WebSocket(`${baseUrl.replace(/^http/, 'ws')}/api/runs/${first.run_id}/stream`);
  const timeout = setTimeout(() => {
    socket.close();
    reject(new Error(`WebSocket messages incomplete: ${[...types].join(',')}`));
  }, 5_000);
  socket.addEventListener('message', (event) => {
    const message = JSON.parse(event.data);
    types.add(message.type);
    if (['snapshot', 'metric_update', 'transaction', 'alert', 'run_status'].every((type) => types.has(type))) {
      clearTimeout(timeout);
      socket.close();
      resolve([...types].sort());
    }
  });
  socket.addEventListener('error', () => {
    clearTimeout(timeout);
    reject(new Error('WebSocket connection failed'));
  });
});
await request(`/api/runs/${first.run_id}/pause`, { method: 'POST' });

console.log(JSON.stringify({
  ok: true,
  runs: [first.run_id, second.run_id],
  comparison_rows: comparison.runs.length,
  filtered_events: events.length,
  filtered_transactions: transactions.length,
  metric_points: metrics.length,
  parquet_magic: magic,
  websocket_types: streamTypes,
}));
