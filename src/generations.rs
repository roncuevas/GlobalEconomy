//! Modo «Generaciones»: una sociedad pequeña que nace, se reproduce, hereda y muere.
//!
//! Es un motor independiente de `Simulation`: la población cambia en cada tick, así que no
//! comparte las suposiciones de población fija del laboratorio. Cada tick representa un año y
//! todo nacimiento, muerte o herencia queda registrado como un evento explicable.

use crate::Strategy;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Sal del RNG para que una semilla no produzca la misma secuencia que el laboratorio.
const RNG_SALT: u64 = 0x6E65_7261_7469_6F6E;
/// Dinero inicial de cada fundador: todos empiezan iguales.
const FOUNDER_MONEY: f64 = 100.0;
/// Edad laboral: solo los adultos de esta franja cobran salario.
const WORK_AGE: (u16, u16) = (18, 64);
/// Salario base anual, multiplicado por la habilidad y una variación aleatoria.
const BASE_WAGE: f64 = 20.0;
/// Costo de vida anual de un adulto y de un menor (lo paga su progenitor).
const ADULT_COST: f64 = 12.0;
const MINOR_COST: f64 = 6.0;
/// Edad fértil, dinero mínimo y máximo de hijos para tener un hijo.
const FERTILE_AGE: (u16, u16) = (20, 45);
const BIRTH_MIN_MONEY: f64 = 60.0;
const MAX_CHILDREN: usize = 4;
/// Probabilidad anual de tener un hijo cuando la sociedad está vacía.
const BIRTH_RATE: f64 = 0.25;
/// Proporción del dinero del progenitor que recibe cada hijo al nacer.
const DOWRY_SHARE: f64 = 0.25;
/// Probabilidad de que un hijo no copie la estrategia de su progenitor.
const STRATEGY_MUTATION: f64 = 0.10;
/// Años seguidos sin cubrir el costo de vida antes de que alguien no pueda sostenerse.
const HARDSHIP_LIMIT: u8 = 5;
/// Ticks que un fallecido sigue visible para que la UI lo desvanezca.
const FADE_TICKS: u64 = 3;
const HISTORY_LIMIT: usize = 1_000;

const SURNAMES: [&str; 60] = [
    "García",
    "Romero",
    "Torres",
    "Herrera",
    "Navarro",
    "Ortega",
    "Castillo",
    "Mendoza",
    "Ruiz",
    "Vargas",
    "Molina",
    "Delgado",
    "Rojas",
    "Campos",
    "Guzmán",
    "Aguilar",
    "Salazar",
    "Pineda",
    "Ibarra",
    "Cordero",
    "Fuentes",
    "Medina",
    "Valdez",
    "Rivas",
    "Paredes",
    "Cabrera",
    "Soto",
    "Luna",
    "Miranda",
    "Acosta",
    "Bravo",
    "Cuevas",
    "Domínguez",
    "Espinoza",
    "Figueroa",
    "Gallardo",
    "Huerta",
    "Iglesias",
    "Juárez",
    "Lara",
    "Maldonado",
    "Nieto",
    "Olmos",
    "Peña",
    "Quiroga",
    "Ríos",
    "Sandoval",
    "Tapia",
    "Ulloa",
    "Vega",
    "Zamora",
    "Arias",
    "Benítez",
    "Carrasco",
    "Duarte",
    "Escobar",
    "Ferrer",
    "Galindo",
    "Hidalgo",
    "Montes",
];
const NAMES: [&str; 40] = [
    "Lucía", "Tomás", "Valeria", "Mateo", "Sofía", "Diego", "Camila", "Emilio", "Renata", "Julián",
    "Elena", "Andrés", "Carmen", "Joaquín", "Inés", "Rafael", "Paula", "Héctor", "Marina",
    "Samuel", "Alba", "Bruno", "Irene", "Gabriel", "Rosa", "Martín", "Clara", "Hugo", "Lola",
    "Óscar", "Ximena", "Iván", "Teresa", "Pablo", "Nora", "Álvaro", "Luz", "Ramón", "Ana",
    "Felipe",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GenConfig {
    pub seed: u64,
    /// Familias fundadoras (2–60).
    pub founders: u32,
    /// Población viva máxima que sostienen los recursos (20–2000).
    pub max_population: u32,
    /// Proporción de cada herencia que pasa al fondo público (0–1).
    pub inheritance_tax: f64,
    /// Rendimiento anual del dinero ahorrado (0–0.10).
    pub capital_return: f64,
    pub max_ticks: u64,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            founders: 12,
            max_population: 300,
            inheritance_tax: 0.0,
            capital_return: 0.04,
            max_ticks: 400,
        }
    }
}

impl GenConfig {
    pub fn sanitized(mut self) -> Self {
        self.founders = self.founders.clamp(2, 60);
        self.max_population = self.max_population.clamp(20, 2_000).max(self.founders);
        self.inheritance_tax = finite_or(self.inheritance_tax, 0.0).clamp(0.0, 1.0);
        self.capital_return = finite_or(self.capital_return, 0.04).clamp(0.0, 0.10);
        self.max_ticks = self.max_ticks.clamp(1, 2_000);
        self
    }
}

fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeathCause {
    OldAge,
    Poverty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Citizen {
    pub id: u32,
    pub name: String,
    pub lineage: u16,
    pub parent: Option<u32>,
    pub generation: u16,
    pub born_tick: u64,
    pub age: u16,
    pub alive: bool,
    pub died_tick: Option<u64>,
    pub cause: Option<DeathCause>,
    pub money: f64,
    pub money_prev: f64,
    pub skill: f64,
    pub strategy: Strategy,
    pub children: Vec<u32>,
    /// Salarios más rendimientos acumulados (netos de pérdidas).
    pub earned_total: f64,
    pub dowry_received: f64,
    pub inherited_total: f64,
    pub transfers_received: f64,
    pub hardship_ticks: u8,
    pub x: f32,
    pub y: f32,
    heading: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lineage {
    pub id: u16,
    pub surname: String,
    pub founder: u32,
    pub alive: u32,
    pub ever: u32,
    pub extinct_tick: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GenEvent {
    Birth {
        tick: u64,
        child: u32,
        parent: u32,
        lineage: u16,
        dowry: f64,
    },
    Death {
        tick: u64,
        id: u32,
        lineage: u16,
        age: u16,
        cause: DeathCause,
        estate: f64,
    },
    Inheritance {
        tick: u64,
        from: u32,
        heirs: Vec<u32>,
        each: f64,
        tax: f64,
        to_state: f64,
    },
    Redistribution {
        tick: u64,
        amount: f64,
        recipients: u32,
    },
    Extinction {
        tick: u64,
        lineage: u16,
    },
    Collapse {
        tick: u64,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GenMetrics {
    pub tick: u64,
    pub population: u32,
    pub births: u32,
    pub deaths: u32,
    pub gini: f64,
    pub top_10_share: f64,
    pub mean_wealth: f64,
    pub median_wealth: f64,
    pub total_wealth: f64,
    pub lineages_alive: u32,
    /// Parte de la riqueza recibida (dotes, herencias y fondo público) frente a lo ganado.
    pub inherited_share: f64,
    pub max_generation: u16,
    pub state_collected: f64,
}

/// Flujos que cruzan la frontera de la sociedad en el tick: el resto son transferencias internas.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GenFlows {
    pub wages: f64,
    pub returns: f64,
    pub consumption: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenSim {
    pub config: GenConfig,
    pub tick: u64,
    pub citizens: Vec<Citizen>,
    pub lineages: Vec<Lineage>,
    pub state_fund: f64,
    pub events: Vec<GenEvent>,
    pub metrics: GenMetrics,
    pub metric_history: Vec<GenMetrics>,
    pub flows: GenFlows,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CitizenView {
    pub id: u32,
    pub name: String,
    pub lineage: u16,
    pub parent: Option<u32>,
    pub generation: u16,
    pub age: u16,
    pub alive: bool,
    pub died_tick: Option<u64>,
    pub cause: Option<DeathCause>,
    pub money: f64,
    pub delta: f64,
    pub earned: f64,
    pub received: f64,
    pub strategy: Strategy,
    pub skill: f64,
    pub children: Vec<u32>,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageView {
    pub id: u16,
    pub surname: String,
    pub alive: u32,
    pub ever: u32,
    pub wealth: f64,
    pub share: f64,
    pub extinct_tick: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenSnapshot {
    pub tick: u64,
    pub config: GenConfig,
    pub config_hash: String,
    pub finished: bool,
    pub citizens: Vec<CitizenView>,
    pub lineages: Vec<LineageView>,
    pub events: Vec<GenEvent>,
    pub metrics: GenMetrics,
    pub metric_history: Vec<GenMetrics>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FamilyView {
    pub person: CitizenView,
    pub surname: String,
    pub parent: Option<CitizenView>,
    pub children: Vec<CitizenView>,
    /// Ancestros desde el progenitor hasta el fundador.
    pub ancestors: Vec<CitizenView>,
}

impl GenSim {
    pub fn new(config: GenConfig) -> Self {
        let config = config.sanitized();
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed ^ RNG_SALT);
        let founders = config.founders as usize;
        let mut citizens = Vec::with_capacity(founders);
        let mut lineages = Vec::with_capacity(founders);
        for i in 0..founders {
            let angle =
                std::f32::consts::TAU * i as f32 / founders as f32 - std::f32::consts::FRAC_PI_2;
            let strategy = match rng.random_range(0..3) {
                0 => Strategy::Saver,
                1 => Strategy::Consumer,
                _ => Strategy::Entrepreneur,
            };
            citizens.push(Citizen {
                id: i as u32,
                name: NAMES[rng.random_range(0..NAMES.len())].into(),
                lineage: i as u16,
                parent: None,
                generation: 0,
                born_tick: 0,
                age: rng.random_range(20..=45),
                alive: true,
                died_tick: None,
                cause: None,
                money: FOUNDER_MONEY,
                money_prev: FOUNDER_MONEY,
                skill: rng.random_range(0.7..1.3),
                strategy,
                children: Vec::new(),
                earned_total: 0.0,
                dowry_received: 0.0,
                inherited_total: 0.0,
                transfers_received: 0.0,
                hardship_ticks: 0,
                x: 0.5 + 0.30 * angle.cos(),
                y: 0.5 + 0.30 * angle.sin(),
                heading: angle,
            });
            lineages.push(Lineage {
                id: i as u16,
                surname: SURNAMES[i % SURNAMES.len()].into(),
                founder: i as u32,
                alive: 1,
                ever: 1,
                extinct_tick: None,
            });
        }
        let mut sim = Self {
            config,
            tick: 0,
            citizens,
            lineages,
            state_fund: 0.0,
            events: Vec::new(),
            metrics: GenMetrics::default(),
            metric_history: Vec::new(),
            flows: GenFlows::default(),
        };
        sim.recalculate_metrics(0, 0, 0.0);
        sim
    }

    pub fn config_hash(&self) -> String {
        let bytes = serde_json::to_vec(&self.config).expect("config serializes");
        format!("{:x}", Sha256::digest(bytes))
    }

    pub fn population(&self) -> u32 {
        self.citizens.iter().filter(|c| c.alive).count() as u32
    }

    pub fn finished(&self) -> bool {
        self.tick >= self.config.max_ticks || self.population() == 0
    }

    /// Avanza un año siguiendo las reglas en orden. Devuelve los eventos del año.
    pub fn step(&mut self) -> Vec<GenEvent> {
        if self.finished() {
            self.events.clear();
            return Vec::new();
        }
        self.tick += 1;
        let tick = self.tick;
        let mut rng = ChaCha8Rng::seed_from_u64(
            self.config.seed ^ RNG_SALT ^ tick.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        );
        let mut events = Vec::new();
        self.flows = GenFlows::default();
        for c in self.citizens.iter_mut().filter(|c| c.alive) {
            c.money_prev = c.money;
            c.age += 1;
        }
        let mut order: Vec<usize> = self
            .citizens
            .iter()
            .filter(|c| c.alive)
            .map(|c| c.id as usize)
            .collect();
        shuffle(&mut order, &mut rng);

        // Ganar: salario por trabajo y rendimiento del dinero según la estrategia.
        let mut wages = vec![0.0; self.citizens.len()];
        for &i in &order {
            let c = &mut self.citizens[i];
            let wage = if (WORK_AGE.0..=WORK_AGE.1).contains(&c.age) {
                BASE_WAGE * c.skill * rng.random_range(0.75..1.25)
            } else {
                0.0
            };
            let rate = match c.strategy {
                Strategy::Saver => self.config.capital_return,
                Strategy::Consumer => self.config.capital_return * 0.25,
                Strategy::Entrepreneur => {
                    self.config.capital_return + rng.random_range(-0.12..0.14)
                }
            };
            let returns = (c.money * rate).max(-c.money);
            c.money += wage + returns;
            c.earned_total += wage + returns;
            wages[i] = wage;
            self.flows.wages += wage;
            self.flows.returns += returns;
        }

        // Costo de vida: cada adulto paga el suyo; el progenitor vivo paga el de sus hijos menores.
        for &i in &order {
            let c = &self.citizens[i];
            if c.age < WORK_AGE.0 {
                let payer = c
                    .parent
                    .filter(|&p| {
                        self.citizens[p as usize].alive && self.citizens[p as usize].money > 0.0
                    })
                    .map(|p| p as usize)
                    .unwrap_or(i);
                let paid = MINOR_COST.min(self.citizens[payer].money);
                self.citizens[payer].money -= paid;
                self.flows.consumption += paid;
                continue;
            }
            let propensity = match c.strategy {
                Strategy::Saver => 0.3,
                Strategy::Entrepreneur => 0.5,
                Strategy::Consumer => 0.8,
            };
            let c = &mut self.citizens[i];
            let base = ADULT_COST.min(c.money);
            c.money -= base;
            if base + 1e-9 < ADULT_COST {
                c.hardship_ticks = c.hardship_ticks.saturating_add(1);
            } else {
                c.hardship_ticks = 0;
            }
            let extra = (propensity * (wages[i] - ADULT_COST).max(0.0)).min(c.money);
            c.money -= extra;
            self.flows.consumption += base + extra;
        }

        // Nacimientos: menos probables cuanto más se acerca la población al tope.
        let mut population = order.len() as u32;
        let mut births = 0;
        for &i in &order {
            if population >= self.config.max_population {
                break;
            }
            let c = &self.citizens[i];
            if !(FERTILE_AGE.0..=FERTILE_AGE.1).contains(&c.age)
                || c.money < BIRTH_MIN_MONEY
                || c.children.len() >= MAX_CHILDREN
            {
                continue;
            }
            let chance = BIRTH_RATE * (1.0 - population as f64 / self.config.max_population as f64);
            if !rng.random_bool(chance.clamp(0.0, 1.0)) {
                continue;
            }
            let child_id = self.citizens.len() as u32;
            let parent = &self.citizens[i];
            let dowry = parent.money * DOWRY_SHARE;
            let k = parent.children.len() as f32;
            let generation = parent.generation + 1;
            let fan = ((k * 137.5) % 100.0 - 50.0).to_radians();
            let heading = parent.heading + fan;
            let distance = 0.075 * 0.85_f32.powi(generation as i32);
            let strategy = if rng.random_bool(STRATEGY_MUTATION) {
                match rng.random_range(0..3) {
                    0 => Strategy::Saver,
                    1 => Strategy::Consumer,
                    _ => Strategy::Entrepreneur,
                }
            } else {
                parent.strategy.clone()
            };
            let child = Citizen {
                id: child_id,
                name: NAMES[rng.random_range(0..NAMES.len())].into(),
                lineage: parent.lineage,
                parent: Some(parent.id),
                generation,
                born_tick: tick,
                age: 0,
                alive: true,
                died_tick: None,
                cause: None,
                money: dowry,
                money_prev: 0.0,
                skill: (0.5 * parent.skill + 0.5 + rng.random_range(-0.15..0.15)).clamp(0.5, 1.6),
                strategy,
                children: Vec::new(),
                earned_total: 0.0,
                dowry_received: dowry,
                inherited_total: 0.0,
                transfers_received: 0.0,
                hardship_ticks: 0,
                x: (parent.x + distance * heading.cos()).clamp(0.03, 0.97),
                y: (parent.y + distance * heading.sin()).clamp(0.03, 0.97),
                heading,
            };
            let lineage = parent.lineage;
            self.citizens[i].money -= dowry;
            self.citizens[i].children.push(child_id);
            self.citizens.push(child);
            self.lineages[lineage as usize].alive += 1;
            self.lineages[lineage as usize].ever += 1;
            population += 1;
            births += 1;
            events.push(GenEvent::Birth {
                tick,
                child: child_id,
                parent: i as u32,
                lineage,
                dowry,
            });
        }

        // Muertes por vejez o por no poder sostenerse; la herencia se reparte de inmediato.
        let mut deaths = 0;
        let mut state_collected = 0.0;
        for &i in &order {
            let c = &self.citizens[i];
            if !c.alive {
                continue;
            }
            let old_age = 0.0005 + 0.00003 * (0.1 * c.age as f64).exp();
            let cause = if c.hardship_ticks >= HARDSHIP_LIMIT {
                Some(DeathCause::Poverty)
            } else if rng.random_bool(old_age.clamp(0.0, 1.0)) {
                Some(DeathCause::OldAge)
            } else {
                None
            };
            let Some(cause) = cause else { continue };
            let estate = c.money;
            let (age, lineage) = (c.age, c.lineage);
            let heirs: Vec<u32> = c
                .children
                .iter()
                .copied()
                .filter(|&h| self.citizens[h as usize].alive)
                .collect();
            let c = &mut self.citizens[i];
            c.alive = false;
            c.died_tick = Some(tick);
            c.cause = Some(cause.clone());
            c.money = 0.0;
            self.lineages[lineage as usize].alive -= 1;
            deaths += 1;
            events.push(GenEvent::Death {
                tick,
                id: i as u32,
                lineage,
                age,
                cause,
                estate,
            });
            if estate <= 0.0 {
                continue;
            }
            let tax = if heirs.is_empty() {
                0.0
            } else {
                estate * self.config.inheritance_tax
            };
            let to_state = if heirs.is_empty() { estate } else { tax };
            let each = if heirs.is_empty() {
                0.0
            } else {
                (estate - tax) / heirs.len() as f64
            };
            for &h in &heirs {
                let heir = &mut self.citizens[h as usize];
                heir.money += each;
                heir.inherited_total += each;
            }
            self.state_fund += to_state;
            state_collected += to_state;
            events.push(GenEvent::Inheritance {
                tick,
                from: i as u32,
                heirs,
                each,
                tax,
                to_state,
            });
        }

        // Redistribución: el fondo público se reparte por igual entre los vivos.
        let living: Vec<usize> = self
            .citizens
            .iter()
            .filter(|c| c.alive)
            .map(|c| c.id as usize)
            .collect();
        if self.state_fund > 0.0 && !living.is_empty() {
            let amount = self.state_fund;
            let share = amount / living.len() as f64;
            for &i in &living {
                self.citizens[i].money += share;
                self.citizens[i].transfers_received += share;
            }
            self.state_fund = 0.0;
            events.push(GenEvent::Redistribution {
                tick,
                amount,
                recipients: living.len() as u32,
            });
        }

        for lineage in &mut self.lineages {
            if lineage.alive == 0 && lineage.extinct_tick.is_none() {
                lineage.extinct_tick = Some(tick);
                events.push(GenEvent::Extinction {
                    tick,
                    lineage: lineage.id,
                });
            }
        }
        if living.is_empty() {
            events.push(GenEvent::Collapse { tick });
        }

        self.recalculate_metrics(births, deaths, state_collected);
        self.events = events.clone();
        events
    }

    fn recalculate_metrics(&mut self, births: u32, deaths: u32, state_collected: f64) {
        let living: Vec<&Citizen> = self.citizens.iter().filter(|c| c.alive).collect();
        let mut wealth: Vec<f64> = living.iter().map(|c| c.money).collect();
        wealth.sort_by(|a, b| a.total_cmp(b));
        let (gini, top_10_share, median_wealth) = distribution(&wealth);
        let total: f64 = wealth.iter().sum();
        let received: f64 = living
            .iter()
            .map(|c| c.dowry_received + c.inherited_total + c.transfers_received)
            .sum();
        let earned: f64 = living.iter().map(|c| c.earned_total.max(0.0)).sum();
        self.metrics = GenMetrics {
            tick: self.tick,
            population: living.len() as u32,
            births,
            deaths,
            gini,
            top_10_share,
            mean_wealth: if living.is_empty() {
                0.0
            } else {
                total / living.len() as f64
            },
            median_wealth,
            total_wealth: total,
            lineages_alive: self.lineages.iter().filter(|l| l.alive > 0).count() as u32,
            inherited_share: if received + earned > 0.0 {
                received / (received + earned)
            } else {
                0.0
            },
            max_generation: living.iter().map(|c| c.generation).max().unwrap_or(0),
            state_collected,
        };
        self.metric_history.push(self.metrics.clone());
        if self.metric_history.len() > HISTORY_LIMIT {
            self.metric_history.remove(0);
        }
    }

    fn view(&self, c: &Citizen) -> CitizenView {
        CitizenView {
            id: c.id,
            name: c.name.clone(),
            lineage: c.lineage,
            parent: c.parent,
            generation: c.generation,
            age: c.age,
            alive: c.alive,
            died_tick: c.died_tick,
            cause: c.cause.clone(),
            money: c.money,
            delta: if c.alive { c.money - c.money_prev } else { 0.0 },
            earned: c.earned_total,
            received: c.dowry_received + c.inherited_total + c.transfers_received,
            strategy: c.strategy.clone(),
            skill: c.skill,
            children: c.children.clone(),
            x: c.x,
            y: c.y,
        }
    }

    pub fn snapshot(&self) -> GenSnapshot {
        let total = self.metrics.total_wealth;
        let mut lineage_wealth = vec![0.0; self.lineages.len()];
        for c in self.citizens.iter().filter(|c| c.alive) {
            lineage_wealth[c.lineage as usize] += c.money;
        }
        GenSnapshot {
            tick: self.tick,
            config: self.config.clone(),
            config_hash: self.config_hash(),
            finished: self.finished(),
            citizens: self
                .citizens
                .iter()
                .filter(|c| {
                    c.alive
                        || c.died_tick
                            .is_some_and(|t| self.tick.saturating_sub(t) < FADE_TICKS)
                })
                .map(|c| self.view(c))
                .collect(),
            lineages: self
                .lineages
                .iter()
                .map(|l| LineageView {
                    id: l.id,
                    surname: l.surname.clone(),
                    alive: l.alive,
                    ever: l.ever,
                    wealth: lineage_wealth[l.id as usize],
                    share: if total > 0.0 {
                        lineage_wealth[l.id as usize] / total
                    } else {
                        0.0
                    },
                    extinct_tick: l.extinct_tick,
                })
                .collect(),
            events: self.events.clone(),
            metrics: self.metrics.clone(),
            metric_history: self
                .metric_history
                .iter()
                .rev()
                .take(200)
                .rev()
                .cloned()
                .collect(),
        }
    }

    pub fn family(&self, id: u32) -> Option<FamilyView> {
        let person = self.citizens.get(id as usize)?;
        let mut ancestors = Vec::new();
        let mut cursor = person.parent;
        while let Some(p) = cursor {
            let ancestor = &self.citizens[p as usize];
            ancestors.push(self.view(ancestor));
            cursor = ancestor.parent;
        }
        Some(FamilyView {
            person: self.view(person),
            surname: self.lineages[person.lineage as usize].surname.clone(),
            parent: person.parent.map(|p| self.view(&self.citizens[p as usize])),
            children: person
                .children
                .iter()
                .map(|&c| self.view(&self.citizens[c as usize]))
                .collect(),
            ancestors,
        })
    }
}

/// Fisher–Yates con el RNG del tick, para que el orden de proceso no favorezca ids bajos.
fn shuffle(items: &mut [usize], rng: &mut ChaCha8Rng) {
    for i in (1..items.len()).rev() {
        let j = rng.random_range(0..=i);
        items.swap(i, j);
    }
}

/// Gini, participación del 10% más rico y mediana de una lista ordenada de menor a mayor.
fn distribution(sorted: &[f64]) -> (f64, f64, f64) {
    let n = sorted.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let total: f64 = sorted.iter().sum();
    let median = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    };
    if total <= 0.0 {
        return (0.0, 0.0, median);
    }
    let weighted: f64 = sorted
        .iter()
        .enumerate()
        .map(|(i, w)| (i as f64 + 1.0) * w)
        .sum();
    let gini =
        ((2.0 * weighted) / (n as f64 * total) - (n as f64 + 1.0) / n as f64).clamp(0.0, 1.0);
    let top = n.div_ceil(10);
    let top_share = sorted[n - top..].iter().sum::<f64>() / total;
    (gini, top_share, median)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(config: GenConfig, ticks: u64) -> GenSim {
        let mut sim = GenSim::new(config);
        for _ in 0..ticks {
            sim.step();
        }
        sim
    }

    fn total_money(sim: &GenSim) -> f64 {
        sim.citizens
            .iter()
            .filter(|c| c.alive)
            .map(|c| c.money)
            .sum::<f64>()
            + sim.state_fund
    }

    #[test]
    fn same_seed_same_society() {
        let a = serde_json::to_string(&run(GenConfig::default(), 200).snapshot()).unwrap();
        let b = serde_json::to_string(&run(GenConfig::default(), 200).snapshot()).unwrap();
        assert_eq!(a, b);
        let c = serde_json::to_string(
            &run(
                GenConfig {
                    seed: 7,
                    ..GenConfig::default()
                },
                200,
            )
            .snapshot(),
        )
        .unwrap();
        assert_ne!(a, c);
    }

    #[test]
    fn population_never_exceeds_cap() {
        for seed in 1..20 {
            for cap in [20, 50, 300] {
                let mut sim = GenSim::new(GenConfig {
                    seed,
                    max_population: cap,
                    ..GenConfig::default()
                });
                for _ in 0..300 {
                    sim.step();
                    assert!(
                        sim.population() <= cap,
                        "seed {seed} cap {cap} tick {}",
                        sim.tick
                    );
                }
            }
        }
    }

    #[test]
    fn money_only_changes_through_wages_returns_and_consumption() {
        for tax in [0.0, 0.5, 1.0] {
            let mut sim = GenSim::new(GenConfig {
                seed: 3,
                inheritance_tax: tax,
                ..GenConfig::default()
            });
            for _ in 0..250 {
                let before = total_money(&sim);
                sim.step();
                let expected = before + sim.flows.wages + sim.flows.returns - sim.flows.consumption;
                let after = total_money(&sim);
                assert!(
                    (after - expected).abs() <= 1e-6 * expected.abs().max(1.0),
                    "tick {} {after} vs {expected}",
                    sim.tick
                );
            }
        }
    }

    #[test]
    fn inheritance_splits_estate_among_living_children_after_tax() {
        let mut sim = GenSim::new(GenConfig {
            seed: 11,
            inheritance_tax: 0.3,
            ..GenConfig::default()
        });
        let mut checked = 0;
        for _ in 0..300 {
            let events = sim.step();
            for event in &events {
                let GenEvent::Death { id, estate, .. } = event else {
                    continue;
                };
                let inheritance = events
                    .iter()
                    .find(|e| matches!(e, GenEvent::Inheritance { from, .. } if from == id));
                if *estate <= 0.0 {
                    assert!(inheritance.is_none());
                    continue;
                }
                let Some(GenEvent::Inheritance {
                    heirs,
                    each,
                    tax,
                    to_state,
                    ..
                }) = inheritance
                else {
                    panic!("missing inheritance for {id}")
                };
                let paid = each * heirs.len() as f64 + to_state;
                assert!((paid - estate).abs() < 1e-6, "estate {estate} paid {paid}");
                if heirs.is_empty() {
                    assert!((to_state - estate).abs() < 1e-9);
                } else {
                    assert!((tax - estate * 0.3).abs() < 1e-9);
                    assert!(heirs
                        .iter()
                        .all(|h| sim.citizens[*h as usize].parent == Some(*id)));
                }
                checked += 1;
            }
        }
        assert!(checked > 10, "only {checked} inheritances observed");
    }

    #[test]
    fn genealogy_is_consistent() {
        let sim = run(
            GenConfig {
                seed: 5,
                ..GenConfig::default()
            },
            250,
        );
        for c in &sim.citizens {
            assert_eq!(
                c.id as usize,
                sim.citizens.iter().position(|x| x.id == c.id).unwrap()
            );
            match c.parent {
                None => {
                    assert_eq!(c.generation, 0);
                    assert_eq!(sim.lineages[c.lineage as usize].founder, c.id);
                }
                Some(p) => {
                    let parent = &sim.citizens[p as usize];
                    assert!(p < c.id);
                    assert_eq!(c.generation, parent.generation + 1);
                    assert_eq!(c.lineage, parent.lineage);
                    assert!(parent.children.contains(&c.id));
                }
            }
        }
        for l in &sim.lineages {
            assert_eq!(
                l.alive as usize,
                sim.citizens
                    .iter()
                    .filter(|c| c.alive && c.lineage == l.id)
                    .count()
            );
        }
    }

    #[test]
    fn positions_stay_in_bounds_and_never_move() {
        let mut sim = run(
            GenConfig {
                seed: 9,
                ..GenConfig::default()
            },
            60,
        );
        let before: Vec<(f32, f32)> = sim.citizens.iter().map(|c| (c.x, c.y)).collect();
        for _ in 0..50 {
            sim.step();
        }
        for (c, (x, y)) in sim.citizens.iter().zip(before) {
            assert_eq!((c.x, c.y), (x, y));
        }
        assert!(sim
            .citizens
            .iter()
            .all(|c| (0.0..=1.0).contains(&c.x) && (0.0..=1.0).contains(&c.y)));
    }

    #[test]
    fn defaults_grow_dynasties_and_inequality() {
        for seed in 1..=10 {
            let sim = run(
                GenConfig {
                    seed,
                    ..GenConfig::default()
                },
                150,
            );
            assert!(sim.population() > 0, "seed {seed} collapsed");
            assert!(
                sim.metrics.gini > 0.25,
                "seed {seed} gini {}",
                sim.metrics.gini
            );
            assert!(
                sim.metrics.max_generation >= 3,
                "seed {seed} generation {}",
                sim.metrics.max_generation
            );
        }
    }

    #[test]
    fn inheritance_tax_reduces_inequality() {
        let mut lower = 0;
        for seed in 1..=10 {
            let untaxed = run(
                GenConfig {
                    seed,
                    ..GenConfig::default()
                },
                150,
            )
            .metrics
            .gini;
            let taxed = run(
                GenConfig {
                    seed,
                    inheritance_tax: 0.8,
                    ..GenConfig::default()
                },
                150,
            )
            .metrics
            .gini;
            if taxed < untaxed {
                lower += 1;
            }
        }
        assert!(lower >= 8, "tax lowered gini in only {lower}/10 seeds");
    }

    #[test]
    fn config_is_sanitized_and_tolerates_missing_fields() {
        let config = GenConfig {
            founders: 500,
            max_population: 5,
            inheritance_tax: 3.0,
            capital_return: f64::NAN,
            ..GenConfig::default()
        }
        .sanitized();
        assert_eq!(config.founders, 60);
        assert_eq!(config.max_population, 60);
        assert_eq!(config.inheritance_tax, 1.0);
        assert_eq!(config.capital_return, 0.04);
        let parsed: GenConfig = serde_json::from_str(r#"{"seed":3,"founders":8}"#).unwrap();
        assert_eq!(parsed.max_ticks, 400);
        assert_eq!(parsed.founders, 8);
    }

    #[test]
    fn simulation_stops_at_max_ticks() {
        let sim = run(
            GenConfig {
                max_ticks: 30,
                ..GenConfig::default()
            },
            50,
        );
        assert_eq!(sim.tick, 30);
        assert!(sim.snapshot().finished);
    }
}
