use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub income_tax_rate: f64,
    pub transfer_per_person: f64,
    pub base_interest_rate: f64,
    #[serde(default)]
    pub liquidity_injection: f64,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            income_tax_rate: 0.18,
            transfer_per_person: 20.0,
            base_interest_rate: 0.04,
            liquidity_injection: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioConfig {
    pub food_price: f64,
    pub wage: f64,
}

impl Default for ScenarioConfig {
    fn default() -> Self {
        Self {
            food_price: 12.0,
            wage: 100.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunConfig {
    pub seed: u64,
    pub model_version: String,
    pub agent_count: usize,
    pub regions: usize,
    pub ticks: u64,
    #[serde(default)]
    pub policy_config: PolicyConfig,
    #[serde(default)]
    pub scenario_config: ScenarioConfig,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            model_version: "0.1.0".into(),
            agent_count: 10_000,
            regions: 8,
            ticks: 1_000,
            policy_config: Default::default(),
            scenario_config: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: u32,
    pub region: u16,
    pub city: u16,
    pub money: f64,
    pub income: f64,
    pub wealth: f64,
    pub debt: f64,
    pub need: f64,
    pub employed: bool,
    pub risk_score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Metrics {
    pub gdp: f64,
    pub gini: f64,
    pub unemployment: f64,
    pub inflation: f64,
    pub tax_revenue: f64,
    pub trade_volume: f64,
    pub alerts: u64,
    pub mean_wealth: f64,
    pub money_supply: f64,
    pub interest_rate: f64,
    pub top_10_wealth_share: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualAgent {
    pub id: u32,
    pub kind: String,
    pub region: u16,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub risk_score: f32,
    pub employed: bool,
    pub wealth: f32,
    pub wealth_delta: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualEdge {
    pub source: u32,
    pub target: u32,
    pub kind: String,
    pub weight: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u64,
    pub agents: Vec<VisualAgent>,
    pub edges: Vec<VisualEdge>,
    pub metrics: Metrics,
    pub policy: PolicyConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomicEvent {
    pub tick: u64,
    pub event_type: String,
    pub agent_id: u32,
    pub amount: f64,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct Simulation {
    pub config: RunConfig,
    pub people: Vec<Person>,
    pub tick: u64,
    pub metrics: Metrics,
    pub events: Vec<EconomicEvent>,
    previous_wealth: Vec<f64>,
}

impl Simulation {
    pub fn new(config: RunConfig) -> Self {
        let count = config.agent_count.min(100_000);
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let mut people = Vec::with_capacity(count);
        for id in 0..count as u32 {
            let region = rng.random_range(0..config.regions.max(1)) as u16;
            let wealth = rng.random_range(500.0..5_000.0);
            people.push(Person {
                id,
                region,
                city: (region * 3 + rng.random_range(0..3)) as u16,
                money: wealth,
                income: 0.0,
                wealth,
                debt: 0.0,
                need: rng.random_range(0.4..1.0),
                employed: rng.random_bool(0.82),
                risk_score: 0.0,
            });
        }
        let mut sim = Self {
            config,
            people,
            tick: 0,
            metrics: Metrics::default(),
            events: Vec::new(),
            previous_wealth: vec![0.0; count],
        };
        sim.recalculate_metrics();
        sim
    }

    pub fn step(&mut self) -> Snapshot {
        if self.tick >= self.config.ticks {
            return self.snapshot();
        }
        self.previous_wealth = self.people.iter().map(|p| p.wealth).collect();
        let seed = self.config.seed ^ self.tick.wrapping_mul(0x9E3779B97F4A7C15);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let wage = self.config.scenario_config.wage;
        let policy_rate = self.config.policy_config.base_interest_rate.clamp(0.0, 1.0);
        let price = self.config.scenario_config.food_price
            * (1.0 + (self.tick as f64 / 100_000.0) + policy_rate * 0.5);
        let tax_rate = self.config.policy_config.income_tax_rate.clamp(0.0, 1.0);
        let transfer = self.config.policy_config.transfer_per_person.max(0.0);
        let liquidity = self.config.policy_config.liquidity_injection.max(0.0);
        let per_person_liquidity = liquidity / self.people.len().max(1) as f64;
        self.events.clear();
        for person in &mut self.people {
            person.income = if person.employed {
                wage * (1.0 - policy_rate * 0.25)
            } else {
                0.0
            };
            person.money += person.income;
            let tax = person.income * tax_rate;
            person.money -= tax;
            person.money += transfer;
            person.money += per_person_liquidity;
            person.need = (person.need + 0.08).min(1.0);
            if person.money >= price && person.need > 0.45 {
                person.money -= price;
                person.need = (person.need - 0.65).max(0.0);
                self.events.push(EconomicEvent {
                    tick: self.tick,
                    event_type: "purchase".into(),
                    agent_id: person.id,
                    amount: price,
                    reason: "basic need and liquidity".into(),
                });
            }
            person.wealth = (person.money - person.debt).max(0.0);
            person.risk_score = ((person.income * 12.0 + person.money) / (person.wealth + 1.0)
                - 1.0)
                .clamp(0.0, 1.0);
            if rng.random_bool(0.01) {
                person.employed = !person.employed;
            }
        }
        self.tick += 1;
        self.recalculate_metrics();
        self.snapshot()
    }

    fn recalculate_metrics(&mut self) {
        let n = self.people.len() as f64;
        if n == 0.0 {
            return;
        }
        let total: f64 = self.people.iter().map(|p| p.wealth).sum();
        let employed = self.people.iter().filter(|p| p.employed).count() as f64;
        let tax_rate = self.config.policy_config.income_tax_rate.clamp(0.0, 1.0);
        let income: f64 = self.people.iter().map(|p| p.income).sum();
        let mut sorted: Vec<f64> = self.people.iter().map(|p| p.wealth).collect();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let weighted: f64 = sorted
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64 + 1.0) * v)
            .sum();
        let gini = if total > 0.0 {
            (2.0 * weighted) / (n * total) - (n + 1.0) / n
        } else {
            0.0
        };
        let top_count = (self.people.len() / 10).max(1);
        let top_wealth: f64 = sorted.iter().rev().take(top_count).sum();
        self.metrics = Metrics {
            gdp: income,
            gini: gini.clamp(0.0, 1.0),
            unemployment: 1.0 - employed / n,
            inflation: self.tick as f64 / 100_000.0,
            tax_revenue: income * tax_rate,
            trade_volume: self.events.iter().map(|e| e.amount).sum(),
            alerts: self.people.iter().filter(|p| p.risk_score > 0.8).count() as u64,
            mean_wealth: total / n,
            money_supply: self.people.iter().map(|p| p.money).sum(),
            interest_rate: self.config.policy_config.base_interest_rate,
            top_10_wealth_share: if total > 0.0 { top_wealth / total } else { 0.0 },
        };
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut agents: Vec<VisualAgent> = self
            .people
            .iter()
            .map(|p| VisualAgent {
                id: p.id,
                kind: "person".into(),
                region: p.region,
                x: ((p.id % 100) as f32 / 100.0),
                y: ((p.id / 100 % 100) as f32 / 100.0),
                size: (p.wealth.max(1.0).sqrt() as f32 / 9.0).clamp(2.5, 14.0),
                risk_score: p.risk_score as f32,
                employed: p.employed,
                wealth: p.wealth as f32,
                wealth_delta: (p.wealth - self.previous_wealth[p.id as usize]) as f32,
            })
            .collect();
        for region in 0..self.config.regions.max(1) as u16 {
            agents.push(VisualAgent {
                id: 1_000_000 + region as u32,
                kind: "market".into(),
                region,
                x: 0.1 + region as f32 * 0.8 / self.config.regions.max(1) as f32,
                y: 0.88,
                size: 12.0,
                risk_score: 0.0,
                employed: true,
                wealth: self.metrics.money_supply as f32,
                wealth_delta: 0.0,
            });
        }
        let mut edges = Vec::with_capacity(self.people.len() * 2);
        for person in &self.people {
            let market_id = 1_000_000 + person.region as u32;
            edges.push(VisualEdge {
                source: person.id,
                target: market_id,
                kind: "market".into(),
                weight: (person.wealth / self.metrics.mean_wealth.max(1.0))
                    .sqrt()
                    .clamp(0.3, 2.5) as f32,
            });
            if person.id + 1 < self.people.len() as u32
                && self.people[(person.id + 1) as usize].region == person.region
            {
                edges.push(VisualEdge {
                    source: person.id,
                    target: person.id + 1,
                    kind: "trade".into(),
                    weight: 0.8,
                });
            }
        }
        Snapshot {
            tick: self.tick,
            agents,
            edges,
            metrics: self.metrics.clone(),
            policy: self.config.policy_config.clone(),
        }
    }

    pub fn apply_policy(&mut self, policy: PolicyConfig) {
        self.config.policy_config = policy;
        self.recalculate_metrics();
    }

    pub fn config_hash(&self) -> String {
        let bytes = serde_json::to_vec(&self.config).expect("config serializes");
        format!("{:x}", Sha256::digest(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config(seed: u64) -> RunConfig {
        RunConfig {
            seed,
            agent_count: 200,
            regions: 4,
            ticks: 10,
            ..Default::default()
        }
    }

    #[test]
    fn same_seed_same_snapshot() {
        let mut a = Simulation::new(config(7));
        let mut b = Simulation::new(config(7));
        for _ in 0..5 {
            a.step();
            b.step();
        }
        assert_eq!(
            serde_json::to_string(&a.snapshot()).unwrap(),
            serde_json::to_string(&b.snapshot()).unwrap()
        );
    }
    #[test]
    fn different_seed_changes_population() {
        let a = Simulation::new(config(7));
        let b = Simulation::new(config(8));
        assert_ne!(a.people[0].wealth, b.people[0].wealth);
    }
    #[test]
    fn gini_is_bounded() {
        let mut s = Simulation::new(config(42));
        s.step();
        assert!((0.0..=1.0).contains(&s.metrics.gini));
    }
    #[test]
    fn no_negative_money() {
        let mut s = Simulation::new(config(42));
        for _ in 0..10 {
            s.step();
        }
        assert!(s.people.iter().all(|p| p.money >= 0.0));
    }
}
