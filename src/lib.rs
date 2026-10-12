use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

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
    #[serde(default = "default_initial_wealth")]
    pub initial_wealth: f64,
    #[serde(default)]
    pub evolution: bool,
    #[serde(default = "default_shock_probability")]
    pub shock_probability: f64,
    #[serde(default = "default_shock_intensity")]
    pub shock_intensity: f64,
    #[serde(default = "default_strategy_mutation_rate")]
    pub strategy_mutation_rate: f64,
}

fn default_initial_wealth() -> f64 {
    2_500.0
}

fn default_shock_probability() -> f64 {
    0.02
}

fn default_shock_intensity() -> f64 {
    0.18
}

fn default_strategy_mutation_rate() -> f64 {
    0.02
}

fn default_company_count() -> usize {
    80
}

fn default_bank_count() -> usize {
    8
}

fn default_sector_count() -> usize {
    6
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstitutionConfig {
    #[serde(default = "default_company_count")]
    pub company_count: usize,
    #[serde(default = "default_bank_count")]
    pub bank_count: usize,
    #[serde(default = "default_sector_count")]
    pub sector_count: usize,
}

impl Default for InstitutionConfig {
    fn default() -> Self {
        Self {
            company_count: default_company_count(),
            bank_count: default_bank_count(),
            sector_count: default_sector_count(),
        }
    }
}

impl Default for ScenarioConfig {
    fn default() -> Self {
        Self {
            food_price: 12.0,
            wage: 100.0,
            initial_wealth: default_initial_wealth(),
            evolution: false,
            shock_probability: default_shock_probability(),
            shock_intensity: default_shock_intensity(),
            strategy_mutation_rate: default_strategy_mutation_rate(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    Manual,
    Automatic,
}

impl Default for PolicyMode {
    fn default() -> Self {
        Self::Automatic
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
    #[serde(default)]
    pub monetary_history: Vec<MonetaryPolicy>,
    #[serde(default = "default_true")]
    pub historical_mode: bool,
    #[serde(default)]
    pub catastrophes: bool,
    #[serde(default)]
    pub policy_mode: PolicyMode,
    #[serde(default)]
    pub institutions: InstitutionConfig,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonetaryPolicy {
    pub apply_tick: u64,
    #[serde(default)]
    pub era: String,
    pub name: String,
    pub description: String,
    pub base_interest_rate: f64,
    pub liquidity_injection: f64,
    #[serde(default)]
    pub government_debt_change: f64,
    #[serde(default)]
    pub private_credit_creation: f64,
    #[serde(default)]
    pub wealth_shock: f64,
    #[serde(default)]
    pub catastrophe: bool,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoneyEvent {
    pub tick: u64,
    pub kind: String,
    pub amount: f64,
    pub reason: String,
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
            monetary_history: Vec::new(),
            historical_mode: true,
            catastrophes: false,
            policy_mode: PolicyMode::Automatic,
            institutions: Default::default(),
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
    #[serde(default)]
    pub strategy: Strategy,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Saver,
    Consumer,
    Entrepreneur,
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Consumer
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalShock {
    pub id: u64,
    pub tick: u64,
    pub kind: String,
    pub region: u16,
    pub intensity: f64,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Person,
    Company,
    Bank,
    Government,
    CentralBank,
    Market,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Company {
    pub id: u32,
    pub region: u16,
    pub sector: u16,
    pub cash: f64,
    pub inventory: f64,
    pub price: f64,
    pub employees: u32,
    pub production_capacity: f64,
    pub sales: f64,
    pub debt: f64,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bank {
    pub id: u32,
    pub region: u16,
    pub deposits: f64,
    pub reserves: f64,
    pub loans: f64,
    pub capital: f64,
    pub liquidity_ratio: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Government {
    pub id: u32,
    pub cash: f64,
    pub tax_revenue: f64,
    pub spending: f64,
    pub debt: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CentralBank {
    pub id: u32,
    pub reserves: f64,
    pub base_rate: f64,
    pub liquidity_injected: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Market {
    pub id: u32,
    pub region: u16,
    pub sector: u16,
    pub supply: f64,
    pub demand: f64,
    pub price: f64,
    pub volume: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub tick: u64,
    pub from: u32,
    pub to: u32,
    pub kind: String,
    pub amount: f64,
    pub region: u16,
    pub reason: String,
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
    pub money_created_this_tick: f64,
    pub cumulative_money_created: f64,
    pub government_debt: f64,
    pub private_debt: f64,
    pub debt_created_this_tick: f64,
    pub monetary_regime: String,
    pub wealth_shock: f64,
    pub adverse_events: u64,
    pub open_risks: u64,
    pub countermeasures: u64,
    pub systemic_risk: f64,
    pub security_level: f64,
    #[serde(default)]
    pub aml_alerts: u64,
    #[serde(default)]
    pub tax_alerts: u64,
    #[serde(default)]
    pub bankruptcies: u64,
    #[serde(default)]
    pub bank_concentration: f64,
    #[serde(default)]
    pub regional_wealth_gap: f64,
    #[serde(default)]
    pub social_mobility: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualAgent {
    pub id: u32,
    pub kind: String,
    pub region: u16,
    #[serde(default)]
    pub sector: u16,
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
    pub monetary_history: Vec<MonetaryPolicy>,
    pub money_events: Vec<MoneyEvent>,
    pub active_policy: Option<MonetaryPolicy>,
    pub next_policy: Option<MonetaryPolicy>,
    pub policy_mode: PolicyMode,
    pub risk_events: Vec<RiskEvent>,
    pub companies: Vec<Company>,
    pub banks: Vec<Bank>,
    pub government: Government,
    pub central_bank: CentralBank,
    pub markets: Vec<Market>,
    pub transactions: Vec<Transaction>,
    #[serde(default)]
    pub compliance_alerts: Vec<ComplianceAlert>,
    #[serde(default)]
    pub external_shocks: Vec<ExternalShock>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricFrame {
    pub tick: u64,
    pub gdp: f64,
    pub gini: f64,
    pub unemployment: f64,
    pub inflation: f64,
    pub money_supply: f64,
    pub government_debt: f64,
    pub private_debt: f64,
    pub mean_wealth: f64,
    pub systemic_risk: f64,
    pub security_level: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskEvent {
    pub id: u64,
    pub policy_tick: u64,
    pub policy_name: String,
    pub category: String,
    pub name: String,
    pub description: String,
    pub detection_reason: String,
    pub severity: f64,
    pub detected_at: u64,
    pub countermeasure_name: String,
    pub countermeasure_description: String,
    pub countermeasure_applied: bool,
    pub before: MetricFrame,
    pub after_policy: Option<MetricFrame>,
    pub after_countermeasure: Option<MetricFrame>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplianceAlert {
    pub id: u64,
    pub tick: u64,
    pub entity_id: u32,
    pub category: String,
    pub severity: f64,
    pub reason: String,
    pub related_transaction_ids: Vec<u64>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomicEvent {
    pub tick: u64,
    pub event_type: String,
    pub agent_id: u32,
    #[serde(default)]
    pub region: u16,
    pub amount: f64,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Simulation {
    pub config: RunConfig,
    pub people: Vec<Person>,
    pub companies: Vec<Company>,
    pub banks: Vec<Bank>,
    pub government: Government,
    pub central_bank: CentralBank,
    pub markets: Vec<Market>,
    pub transactions: Vec<Transaction>,
    pub tick: u64,
    pub metrics: Metrics,
    pub events: Vec<EconomicEvent>,
    previous_wealth: Vec<f64>,
    money_created_total: f64,
    money_created_this_tick: f64,
    money_events: Vec<MoneyEvent>,
    government_debt: f64,
    private_debt: f64,
    debt_created_this_tick: f64,
    wealth_shock_this_tick: f64,
    last_policy_apply_tick: u64,
    pending_policy: Option<MonetaryPolicy>,
    risk_events: Vec<RiskEvent>,
    systemic_risk: f64,
    security_level: f64,
    next_transaction_id: u64,
    #[serde(default)]
    compliance_alerts: Vec<ComplianceAlert>,
    #[serde(default)]
    external_shocks: Vec<ExternalShock>,
}

impl Simulation {
    pub fn new(config: RunConfig) -> Self {
        let count = config.agent_count.min(100_000);
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let mut people = Vec::with_capacity(count);
        let initial_wealth = config.scenario_config.initial_wealth.max(1.0);
        for id in 0..count as u32 {
            let region = rng.random_range(0..config.regions.max(1)) as u16;
            let wealth = initial_wealth;
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
                strategy: match id % 3 {
                    0 => Strategy::Saver,
                    1 => Strategy::Consumer,
                    _ => Strategy::Entrepreneur,
                },
            });
        }
        let mut config = config;
        if config.monetary_history.is_empty() {
            let history = Self::default_monetary_history(config.ticks);
            config.monetary_history = if config.historical_mode {
                history
            } else {
                history.into_iter().take(1).collect()
            };
        }
        let initial_money = initial_wealth * count as f64;
        let company_count = config.institutions.company_count.max(1);
        let bank_count = config.institutions.bank_count.max(1);
        let sector_count = config.institutions.sector_count.max(1);
        let market_price = config.scenario_config.food_price;
        let companies = (0..company_count)
            .map(|index| Company {
                id: 3_000_000 + index as u32,
                region: (index % config.regions.max(1)) as u16,
                sector: (index % sector_count) as u16,
                cash: initial_wealth * 20.0,
                inventory: 0.0,
                price: config.scenario_config.food_price,
                employees: 0,
                production_capacity: 100.0,
                sales: 0.0,
                debt: 0.0,
                active: true,
            })
            .collect();
        let banks = (0..bank_count)
            .map(|index| {
                let deposits = initial_money / bank_count as f64;
                Bank {
                    id: 4_000_000 + index as u32,
                    region: (index % config.regions.max(1)) as u16,
                    deposits,
                    reserves: deposits * 0.2,
                    loans: 0.0,
                    capital: deposits * 0.08,
                    liquidity_ratio: 0.2,
                }
            })
            .collect();
        let markets = (0..config.regions.max(1))
            .flat_map(|region| {
                (0..sector_count).map(move |sector| Market {
                    id: 1_000_000 + region as u32 * 1_000 + sector as u32,
                    region: region as u16,
                    sector: sector as u16,
                    supply: 0.0,
                    demand: 0.0,
                    price: market_price,
                    volume: 0.0,
                })
            })
            .collect();
        let mut sim = Self {
            config,
            people,
            companies,
            banks,
            government: Government {
                id: 5_000_000,
                cash: 0.0,
                tax_revenue: 0.0,
                spending: 0.0,
                debt: 0.0,
            },
            central_bank: CentralBank {
                id: 6_000_000,
                reserves: initial_money * 0.2,
                base_rate: 0.04,
                liquidity_injected: 0.0,
            },
            markets,
            transactions: Vec::new(),
            tick: 0,
            metrics: Metrics::default(),
            events: Vec::new(),
            previous_wealth: vec![0.0; count],
            money_created_total: initial_money,
            money_created_this_tick: initial_money,
            money_events: vec![MoneyEvent {
                tick: 0,
                kind: "genesis".into(),
                amount: initial_money,
                reason: "dotación inicial igual para todos los agentes".into(),
            }],
            government_debt: 0.0,
            private_debt: 0.0,
            debt_created_this_tick: 0.0,
            wealth_shock_this_tick: 0.0,
            last_policy_apply_tick: 0,
            pending_policy: None,
            risk_events: Vec::new(),
            systemic_risk: 0.0,
            security_level: 0.25,
            next_transaction_id: 0,
            compliance_alerts: Vec::new(),
            external_shocks: Vec::new(),
        };
        sim.recalculate_metrics();
        sim
    }

    pub fn default_monetary_history(_ticks: u64) -> Vec<MonetaryPolicy> {
        let p = |tick: u64,
                 era: &str,
                 name: &str,
                 description: &str,
                 rate: f64,
                 liquidity: f64,
                 government_debt: f64,
                 private_credit: f64,
                 catastrophe: bool,
                 optional: bool| MonetaryPolicy {
            apply_tick: tick,
            era: era.into(),
            name: name.into(),
            description: description.into(),
            base_interest_rate: rate,
            liquidity_injection: liquidity,
            government_debt_change: government_debt,
            private_credit_creation: private_credit,
            wealth_shock: 0.0,
            catastrophe,
            optional,
        };
        let mut history = vec![
            p(
                0,
                "commodity",
                "Génesis monetaria",
                "Todos reciben la misma dotación inicial",
                0.04,
                0.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                2,
                "commodity",
                "Trueque y unidad de cuenta",
                "La comunidad acuerda una unidad para valorar intercambios",
                0.04,
                0.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                4,
                "metallic",
                "Moneda metálica",
                "El Estado acuña piezas con valor y peso reconocibles",
                0.04,
                0.0,
                50_000.0,
                0.0,
                false,
                false,
            ),
            p(
                6,
                "metallic",
                "Impuesto de acuñación",
                "La autoridad obtiene ingresos por emitir moneda",
                0.04,
                0.0,
                20_000.0,
                0.0,
                false,
                false,
            ),
            p(
                8,
                "convertible",
                "Papel convertible",
                "Los recibos representan reservas metálicas depositadas",
                0.035,
                0.0,
                80_000.0,
                0.0,
                false,
                false,
            ),
            p(
                10,
                "credit",
                "Banca y reservas",
                "Los bancos empiezan a crear depósitos al conceder crédito",
                0.035,
                0.0,
                0.0,
                180_000.0,
                false,
                false,
            ),
            p(
                12,
                "credit",
                "Expansión de crédito",
                "Más préstamos amplían depósitos y actividad económica",
                0.025,
                350_000.0,
                0.0,
                300_000.0,
                false,
                false,
            ),
            p(
                14,
                "gold_standard",
                "Patrón oro",
                "La moneda queda convertible a oro a una paridad fija",
                0.05,
                0.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                16,
                "gold_standard",
                "Disciplina externa",
                "La salida de oro fuerza una política más restrictiva",
                0.075,
                0.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                18,
                "war_finance",
                "Financiación de guerra",
                "El gobierno emite deuda para financiar gasto extraordinario",
                0.06,
                0.0,
                500_000.0,
                0.0,
                false,
                false,
            ),
            p(
                20,
                "war_finance",
                "Suspensión de convertibilidad",
                "La convertibilidad se suspende ante la emergencia",
                0.045,
                100_000.0,
                250_000.0,
                0.0,
                false,
                false,
            ),
            p(
                22,
                "gold_exchange",
                "Retorno al oro",
                "Se intenta restaurar la convertibilidad con una paridad modificada",
                0.06,
                0.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                24,
                "depression",
                "Contracción deflacionaria",
                "La defensa de la paridad reduce crédito y demanda",
                0.10,
                0.0,
                0.0,
                -100_000.0,
                false,
                false,
            ),
            p(
                26,
                "depression",
                "Crisis bancaria",
                "Quiebras destruyen depósitos y liquidan crédito",
                0.08,
                0.0,
                0.0,
                -250_000.0,
                true,
                true,
            ),
            p(
                28,
                "devaluation",
                "Devaluación",
                "La autoridad abandona una paridad para recuperar actividad",
                0.04,
                120_000.0,
                100_000.0,
                0.0,
                false,
                false,
            ),
            p(
                30,
                "bretton_woods",
                "Bretton Woods",
                "Tipos de cambio fijos pero ajustables y dólar ligado al oro",
                0.045,
                0.0,
                300_000.0,
                0.0,
                false,
                false,
            ),
            p(
                32,
                "bretton_woods",
                "Reservas en dólares",
                "Los déficits externos expanden reservas y dinero en otros países",
                0.035,
                220_000.0,
                150_000.0,
                0.0,
                false,
                false,
            ),
            p(
                34,
                "fiat_transition",
                "Fin de la ventana de oro",
                "El dólar deja de ser convertible a oro y comienza el dinero fiduciario",
                0.055,
                0.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                36,
                "fiat",
                "Dinero fiduciario",
                "La confianza y la política monetaria sostienen la unidad de cuenta",
                0.06,
                100_000.0,
                200_000.0,
                100_000.0,
                false,
                false,
            ),
            p(
                38,
                "inflation",
                "Gran inflación",
                "El exceso de demanda y expectativas elevan los precios",
                0.08,
                0.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                40,
                "disinflation",
                "Desinflación",
                "Una tasa alta enfría crédito, salarios y precios",
                0.14,
                0.0,
                0.0,
                -80_000.0,
                false,
                false,
            ),
            p(
                42,
                "financialization",
                "Liberalización financiera",
                "Más intermediación amplía el crédito privado y el riesgo",
                0.07,
                0.0,
                0.0,
                450_000.0,
                false,
                false,
            ),
            p(
                44,
                "housing_bubble",
                "Burbuja inmobiliaria",
                "El crédito barato infla precios de activos y garantías",
                0.025,
                180_000.0,
                0.0,
                700_000.0,
                true,
                true,
            ),
            p(
                46,
                "financial_crisis",
                "Estallido inmobiliario",
                "La caída de garantías destruye crédito y riqueza",
                0.09,
                0.0,
                0.0,
                -500_000.0,
                true,
                true,
            ),
            p(
                48,
                "qe",
                "Flexibilización cuantitativa I",
                "El banco central compra activos y crea reservas",
                0.01,
                800_000.0,
                600_000.0,
                0.0,
                false,
                false,
            ),
            p(
                50,
                "sovereign_debt",
                "Crisis de deuda soberana",
                "El gobierno refinancia deuda y aplica consolidación fiscal",
                0.035,
                150_000.0,
                -100_000.0,
                0.0,
                true,
                true,
            ),
            p(
                52,
                "zero_lower_bound",
                "Tasa cercana a cero",
                "La política convencional pierde margen y busca estimular demanda",
                0.005,
                250_000.0,
                250_000.0,
                300_000.0,
                false,
                false,
            ),
            p(
                54,
                "pandemic",
                "Shock sanitario",
                "Una catástrofe reduce producción y activa gasto de emergencia",
                0.01,
                900_000.0,
                800_000.0,
                500_000.0,
                true,
                true,
            ),
            p(
                56,
                "inflation_shock",
                "Shock de oferta",
                "La recuperación y restricciones elevan precios",
                0.07,
                0.0,
                0.0,
                0.0,
                true,
                true,
            ),
            p(
                58,
                "quantitative_tightening",
                "Reducción de balance",
                "Las ventas de activos retiran liquidez gradualmente",
                0.085,
                -300_000.0,
                0.0,
                -120_000.0,
                false,
                false,
            ),
            p(
                60,
                "digital_money",
                "Dinero digital",
                "Los depósitos digitales dominan el sistema de pagos",
                0.05,
                100_000.0,
                0.0,
                180_000.0,
                false,
                false,
            ),
            p(
                62,
                "reserve_asset",
                "Derechos Especiales de Giro",
                "El FMI crea un activo de reserva internacional para complementar oro y dólares",
                0.05,
                90_000.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                64,
                "bretton_woods_reform",
                "Acuerdo Smithsonian",
                "Se reajustan paridades y el precio oficial del oro antes del abandono del sistema fijo",
                0.055,
                40_000.0,
                100_000.0,
                0.0,
                false,
                false,
            ),
            p(
                66,
                "oil_shock",
                "Primer shock petrolero",
                "El embargo de 1973 eleva costes, inflación y necesidad de financiación",
                0.09,
                0.0,
                180_000.0,
                0.0,
                true,
                true,
            ),
            p(
                68,
                "floating_exchange",
                "Tipos de cambio flotantes",
                "Las monedas dejan de defender una paridad fija y absorben shocks mediante el tipo de cambio",
                0.075,
                0.0,
                0.0,
                0.0,
                false,
                false,
            ),
            p(
                70,
                "oil_shock",
                "Segundo shock petrolero",
                "La energía vuelve a presionar precios y actividad a finales de los setenta",
                0.12,
                0.0,
                120_000.0,
                -60_000.0,
                true,
                true,
            ),
            p(
                72,
                "monetarism",
                "Objetivos de agregados monetarios",
                "La autoridad intenta controlar la inflación mediante reservas y crecimiento monetario",
                0.15,
                -80_000.0,
                0.0,
                -80_000.0,
                false,
                false,
            ),
            p(
                74,
                "credit_controls",
                "Controles de crédito",
                "Se restringe la expansión de préstamos para romper la inercia inflacionaria",
                0.16,
                0.0,
                0.0,
                -120_000.0,
                false,
                false,
            ),
            p(
                76,
                "financial_liberalization",
                "Desregulación financiera",
                "La liberalización amplía la intermediación, la competencia y la exposición al riesgo",
                0.09,
                0.0,
                0.0,
                380_000.0,
                false,
                false,
            ),
            p(
                78,
                "exchange_coordination",
                "Acuerdo del Plaza",
                "Las grandes economías coordinan una depreciación del dólar y reequilibrio externo",
                0.07,
                60_000.0,
                80_000.0,
                80_000.0,
                false,
                false,
            ),
            p(
                80,
                "exchange_coordination",
                "Acuerdo del Louvre",
                "Se intenta estabilizar las monedas tras la depreciación coordinada del dólar",
                0.065,
                20_000.0,
                60_000.0,
                40_000.0,
                false,
                false,
            ),
            p(
                82,
                "exchange_crisis",
                "Crisis del Mecanismo Europeo de Cambio",
                "La presión especulativa obliga a devaluar o abandonar bandas cambiarias",
                0.13,
                0.0,
                0.0,
                -160_000.0,
                true,
                true,
            ),
            p(
                84,
                "inflation_targeting",
                "Metas explícitas de inflación",
                "La credibilidad se ancla comunicando un objetivo de inflación verificable",
                0.06,
                30_000.0,
                0.0,
                60_000.0,
                false,
                false,
            ),
            p(
                86,
                "european_union",
                "Tratado de Maastricht",
                "La unión monetaria europea fija criterios de convergencia y disciplina fiscal",
                0.055,
                0.0,
                -60_000.0,
                40_000.0,
                false,
                false,
            ),
            p(
                88,
                "asian_crisis",
                "Crisis financiera asiática",
                "La fuga de capitales y la defensa cambiaria provocan contracción de crédito regional",
                0.14,
                0.0,
                0.0,
                -300_000.0,
                true,
                true,
            ),
            p(
                90,
                "euro",
                "Nacimiento del euro",
                "Once países transfieren la política monetaria a una autoridad supranacional común",
                0.045,
                100_000.0,
                100_000.0,
                120_000.0,
                false,
                false,
            ),
            p(
                92,
                "payment_innovation",
                "Pagos electrónicos masivos",
                "Tarjetas, cajeros y redes electrónicas reducen la fricción de los depósitos bancarios",
                0.04,
                80_000.0,
                0.0,
                160_000.0,
                false,
                false,
            ),
            p(
                94,
                "dotcom_bubble",
                "Burbuja puntocom",
                "La liquidez y expectativas sobre tecnología inflan activos antes de una corrección",
                0.035,
                120_000.0,
                0.0,
                350_000.0,
                true,
                true,
            ),
            p(
                96,
                "japan_zero_rate",
                "Tipo cero en Japón",
                "La autoridad lleva el tipo de corto plazo a cero ante deflación persistente",
                0.001,
                180_000.0,
                100_000.0,
                120_000.0,
                false,
                false,
            ),
            p(
                98,
                "japan_ycc",
                "Control de curva de rendimientos",
                "La autoridad guía también los tipos largos mediante compras de activos y señalización",
                0.002,
                220_000.0,
                120_000.0,
                180_000.0,
                false,
                false,
            ),
            p(
                100,
                "housing_credit",
                "Expansión del crédito hipotecario",
                "La titulización conecta hipotecas con mercados de capital y amplifica el apalancamiento",
                0.025,
                160_000.0,
                0.0,
                520_000.0,
                false,
                false,
            ),
            p(
                102,
                "financial_crisis",
                "Crisis financiera global",
                "El crédito interbancario se congela y las pérdidas de activos destruyen dinero bancario",
                0.11,
                -200_000.0,
                400_000.0,
                -700_000.0,
                true,
                true,
            ),
            p(
                104,
                "qe",
                "Flexibilización cuantitativa II",
                "El banco central compra deuda pública y privada para restaurar liquidez y demanda",
                0.01,
                900_000.0,
                500_000.0,
                0.0,
                false,
                false,
            ),
            p(
                106,
                "euro_crisis",
                "LTRO del Eurosistema",
                "Préstamos de largo plazo a bancos sustituyen mercados mayoristas cerrados",
                0.012,
                480_000.0,
                180_000.0,
                240_000.0,
                false,
                false,
            ),
            p(
                108,
                "euro_crisis",
                "OMT y promesa de hacer lo necesario",
                "Una promesa de compras condicionadas reduce el riesgo de ruptura monetaria",
                0.008,
                320_000.0,
                120_000.0,
                140_000.0,
                false,
                false,
            ),
            p(
                110,
                "negative_rates",
                "Tipos de interés negativos",
                "La tasa de depósito bajo cero cobra por reservas para estimular crédito y precios",
                -0.01,
                260_000.0,
                0.0,
                220_000.0,
                false,
                false,
            ),
            p(
                112,
                "forward_guidance",
                "Orientación futura",
                "La autoridad usa comunicación sobre la trayectoria futura para mover expectativas hoy",
                0.005,
                180_000.0,
                0.0,
                160_000.0,
                false,
                false,
            ),
            p(
                114,
                "crypto_assets",
                "Criptoactivos y dinero programable",
                "Aparece una clase de activos digitales fuera del balance de la banca tradicional",
                0.04,
                40_000.0,
                0.0,
                90_000.0,
                false,
                true,
            ),
            p(
                116,
                "stablecoins",
                "Monedas estables privadas",
                "Pasivos digitales prometen mantener paridad con una moneda o reserva de activos",
                0.045,
                70_000.0,
                0.0,
                130_000.0,
                false,
                true,
            ),
            p(
                118,
                "pandemic_response",
                "Compras de emergencia pandémicas",
                "El banco central compra activos a gran escala mientras el gobierno sostiene ingresos",
                0.005,
                1_100_000.0,
                900_000.0,
                600_000.0,
                true,
                true,
            ),
            p(
                120,
                "fiscal_monetary",
                "Transferencias financiadas durante la pandemia",
                "La coordinación fiscal-monetaria evita una caída abrupta de liquidez y empleo",
                0.01,
                650_000.0,
                1_200_000.0,
                300_000.0,
                false,
                true,
            ),
            p(
                122,
                "inflation_shock",
                "Inflación de reapertura",
                "Cuellos de botella, energía y demanda acumulada reabren la discusión sobre retirar estímulo",
                0.08,
                0.0,
                0.0,
                -100_000.0,
                true,
                true,
            ),
            p(
                124,
                "central_bank_independence",
                "Independencia y meta de estabilidad",
                "El mandato y la autonomía buscan separar la emisión de decisiones fiscales de corto plazo",
                0.06,
                50_000.0,
                0.0,
                40_000.0,
                false,
                false,
            ),
            p(
                126,
                "cbdc",
                "Investigación de moneda digital de banco central",
                "Una CBDC explora dinero público digital complementario a efectivo y depósitos",
                0.045,
                100_000.0,
                0.0,
                80_000.0,
                false,
                true,
            ),
            p(
                128,
                "quantitative_tightening",
                "Normalización y reducción de balance",
                "Las reinversiones se detienen y las ventas de activos retiran reservas del sistema",
                0.09,
                -420_000.0,
                -80_000.0,
                -180_000.0,
                false,
                false,
            ),
            p(
                130,
                "instant_payments",
                "Pagos instantáneos interoperables",
                "La liquidación casi inmediata reduce el tiempo entre depósitos, crédito y comercio",
                0.045,
                80_000.0,
                0.0,
                100_000.0,
                false,
                true,
            ),
            p(
                132,
                "reserve_remuneration",
                "Remuneración de reservas",
                "El banco central paga intereses sobre reservas para controlar el tipo de mercado",
                0.055,
                40_000.0,
                0.0,
                50_000.0,
                false,
                false,
            ),
            p(
                134,
                "macroprudential",
                "Política macroprudencial",
                "Capital, liquidez y límites de préstamo intentan contener riesgos antes de la crisis",
                0.065,
                0.0,
                0.0,
                -140_000.0,
                false,
                false,
            ),
            p(
                136,
                "debt_restructuring",
                "Reestructuración de deuda soberana",
                "Una extensión de vencimientos reduce el servicio inmediato y redistribuye pérdidas",
                0.05,
                0.0,
                -300_000.0,
                0.0,
                true,
                true,
            ),
        ];
        for event in &mut history {
            event.wealth_shock = match event.name.as_str() {
                "Burbuja inmobiliaria" => 0.18,
                "Estallido inmobiliario" | "Crisis financiera global" => -0.42,
                "Crisis de deuda soberana" => -0.18,
                "Shock sanitario" | "Compras de emergencia pandémicas" => -0.30,
                "Shock de oferta" | "Inflación de reapertura" => -0.15,
                "Crisis financiera asiática" | "Crisis del Mecanismo Europeo de Cambio" => -0.12,
                "Burbuja puntocom" => -0.08,
                _ => event.wealth_shock,
            };
        }
        history
    }

    fn metric_frame(&self) -> MetricFrame {
        MetricFrame {
            tick: self.tick,
            gdp: self.metrics.gdp,
            gini: self.metrics.gini,
            unemployment: self.metrics.unemployment,
            inflation: self.metrics.inflation,
            money_supply: self.metrics.money_supply,
            government_debt: self.metrics.government_debt,
            private_debt: self.metrics.private_debt,
            mean_wealth: self.metrics.mean_wealth,
            systemic_risk: self.systemic_risk,
            security_level: self.security_level,
        }
    }

    fn risk_profile(policy: &MonetaryPolicy) -> (String, String, String, String, String, String) {
        let name = policy.name.as_str();
        if name.contains("Papel") || name.contains("Dinero") || name.contains("billete") {
            return (
                "currency_security".into(),
                "Falsificación y pérdida de confianza".into(),
                "Aumenta la circulación de piezas cuya autenticidad no puede verificarse con facilidad; el problema se observa como presión sobre la confianza y el dinero aceptado.".into(),
                "La presión de falsificación y la caída de confianza superan el umbral de alerta.".into(),
                "Blindaje de emisión".into(),
                "Renovar diseños, autenticación pública y trazabilidad de emisiones para proteger la confianza.".into(),
            );
        }
        if name.contains("Banca")
            || name.contains("crédito")
            || name.contains("LTRO")
            || name.contains("reservas")
        {
            return (
                "bank_run".into(),
                "Corrida bancaria y descalce de liquidez".into(),
                "Los retiros potenciales crecen más rápido que las reservas líquidas y el crédito se concentra en pocos intermediarios.".into(),
                "La liquidez disponible queda por debajo de las obligaciones de corto plazo y sube el riesgo sistémico.".into(),
                "Prestamista de última instancia".into(),
                "Abrir liquidez de emergencia contra garantías, reforzar reservas y publicar reglas de acceso.".into(),
            );
        }
        if name.contains("oro") || name.contains("convertibilidad") || name.contains("Smithsonian")
        {
            return (
                "reserve_drain".into(),
                "Fuga de reservas y presión sobre la paridad".into(),
                "La paridad fija pierde credibilidad cuando las reservas no alcanzan para sostener todos los canjes prometidos.".into(),
                "La deuda y los pagos externos crecen mientras cae el colchón de reservas.".into(),
                "Defensa de reservas".into(),
                "Ajustar temporalmente la liquidez, transparentar reservas y permitir una paridad sostenible.".into(),
            );
        }
        if name.contains("petróleo")
            || name.contains("Inflación")
            || name.contains("Gran inflación")
        {
            return (
                "supply_inflation".into(),
                "Shock de oferta e inflación desanclada".into(),
                "Un aumento de costes reduce la producción mientras hogares y empresas anticipan que los precios seguirán subiendo.".into(),
                "La inflación observada y esperada sube por encima de la estabilidad del escenario.".into(),
                "Ancla antiinflacionaria".into(),
                "Combinar comunicación creíble, apoyo focalizado y una normalización gradual que no destruya el empleo.".into(),
            );
        }
        if name.contains("Burbuja") || name.contains("hipotecario") || name.contains("titulización")
        {
            return (
                "asset_bubble".into(),
                "Burbuja de activos y apalancamiento".into(),
                "El crédito barato y las expectativas de revalorización hacen crecer los activos más rápido que los ingresos que los respaldan.".into(),
                "La deuda privada y los precios de activos crecen juntos hasta elevar la vulnerabilidad.".into(),
                "Freno macroprudencial".into(),
                "Elevar colchones de capital, limitar concentración y someter el crédito a pruebas de estrés.".into(),
            );
        }
        if name.contains("cambi")
            || name.contains("Plaza")
            || name.contains("Louvre")
            || name.contains("ERM")
            || name.contains("flotantes")
        {
            return (
                "exchange_attack".into(),
                "Ataque especulativo y tensión cambiaria".into(),
                "Los flujos buscan anticiparse a una devaluación o a un cambio de régimen y amplifican la salida de capitales.".into(),
                "La brecha entre la paridad administrada y los flujos financieros dispara la alerta cambiaria.".into(),
                "Cortafuegos cambiario".into(),
                "Usar reservas, líneas de liquidez y una comunicación coordinada para evitar ajustes desordenados.".into(),
            );
        }
        if name.contains("deuda") || name.contains("reestructuración") || name.contains("soberana")
        {
            return (
                "sovereign_stress".into(),
                "Estrés de deuda soberana".into(),
                "El servicio de deuda y el refinanciamiento consumen recursos mientras un shock reduce ingresos públicos.".into(),
                "La deuda aumenta más rápido que los ingresos y se acorta el plazo de refinanciamiento.".into(),
                "Reperfilamiento preventivo".into(),
                "Extender vencimientos, proteger gasto esencial y negociar antes del incumplimiento.".into(),
            );
        }
        if name.contains("Crisis") || name.contains("Estallido") || name.contains("Recesión") {
            return (
                "liquidity_freeze".into(),
                "Congelamiento de liquidez".into(),
                "La desconfianza hace que los intermediarios retengan efectivo y reduzcan transacciones aun cuando existan activos.".into(),
                "El volumen comercial cae y el riesgo sistémico sube durante varios ticks consecutivos.".into(),
                "Facilidad de liquidez".into(),
                "Restaurar pagos esenciales con liquidez temporal, garantías transparentes y resolución ordenada.".into(),
            );
        }
        if name.contains("CBDC")
            || name.contains("Pagos")
            || name.contains("electrónicos")
            || name.contains("stablecoins")
            || name.contains("Cripto")
        {
            return (
                "payment_integrity".into(),
                "Fraude y fragmentación de pagos".into(),
                "La innovación incorpora nuevos intermediarios y fallos de autenticación que pueden fragmentar la unidad de cuenta.".into(),
                "Aumentan las operaciones anómalas y la diferencia entre dinero aceptado y dinero declarado.".into(),
                "Estándar de pagos seguros".into(),
                "Exigir interoperabilidad, autenticación, reglas de respaldo y límites de concentración.".into(),
            );
        }
        (
            "confidence_shock".into(),
            "Pérdida de confianza monetaria".into(),
            "La comunidad empieza a preferir liquidez inmediata y reduce la aceptación de la unidad monetaria.".into(),
            "Suben simultáneamente la volatilidad, el riesgo y la demanda precautoria de dinero.".into(),
            "Refuerzo de credibilidad".into(),
            "Publicar reglas verificables, estabilizar expectativas y proteger la liquidación de pagos.".into(),
        )
    }

    fn detect_upcoming_risk(&mut self) {
        let Some(policy) = self
            .config
            .monetary_history
            .iter()
            .find(|policy| {
                policy.apply_tick > self.last_policy_apply_tick
                    && (!policy.optional || self.config.catastrophes)
                    && (self.tick + 1 >= policy.apply_tick.saturating_sub(1)
                        || matches!(self.config.policy_mode, PolicyMode::Manual))
            })
            .cloned()
        else {
            return;
        };
        if self
            .risk_events
            .iter()
            .any(|event| event.policy_tick == policy.apply_tick)
        {
            return;
        }
        let (
            category,
            name,
            description,
            detection_reason,
            countermeasure_name,
            countermeasure_description,
        ) = Self::risk_profile(&policy);
        // El riesgo es una señal de presión vigente, no un contador acumulativo.
        // Decaerlo ligeramente evita que una corrida larga quede saturada en 100%
        // solo por haber atravesado muchas políticas históricas.
        let pressure = 0.015
            + policy.private_credit_creation.abs() / 20_000_000.0
            + policy.government_debt_change.abs() / 50_000_000.0;
        self.systemic_risk = (self.systemic_risk * 0.92 + pressure).clamp(0.0, 1.0);
        self.risk_events.push(RiskEvent {
            id: self.risk_events.len() as u64,
            policy_tick: policy.apply_tick,
            policy_name: policy.name,
            category,
            name,
            description,
            detection_reason,
            severity: (0.25 + self.systemic_risk * 0.65).clamp(0.0, 1.0),
            detected_at: self.tick,
            countermeasure_name,
            countermeasure_description,
            countermeasure_applied: false,
            before: self.metric_frame(),
            after_policy: None,
            after_countermeasure: None,
        });
    }

    pub fn apply_countermeasure(&mut self) -> bool {
        let Some(index) = self
            .risk_events
            .iter()
            .rposition(|event| !event.countermeasure_applied)
        else {
            return false;
        };
        let category = self.risk_events[index].category.clone();
        if category == "currency_security" || category == "payment_integrity" {
            self.security_level = (self.security_level + 0.22).min(1.0);
        }
        let relief = match category.as_str() {
            "sovereign_stress" => 0.32,
            "bank_run" | "liquidity_freeze" => 0.28,
            "asset_bubble" | "exchange_attack" | "reserve_drain" => 0.24,
            _ => 0.18,
        };
        self.systemic_risk = (self.systemic_risk - relief).max(0.0);
        let after = self.metric_frame();
        self.risk_events[index].countermeasure_applied = true;
        self.risk_events[index].after_countermeasure = Some(after);
        self.recalculate_metrics();
        true
    }

    pub fn resolve_compliance_alert(&mut self, alert_id: u64) -> bool {
        let Some(alert) = self
            .compliance_alerts
            .iter_mut()
            .find(|alert| alert.id == alert_id && alert.status == "open")
        else {
            return false;
        };
        alert.status = "resolved".into();
        self.recalculate_metrics();
        true
    }

    fn apply_policy_event(&mut self, policy: &MonetaryPolicy) {
        self.config.policy_config.base_interest_rate = policy.base_interest_rate;
        self.config.policy_config.liquidity_injection = policy.liquidity_injection;
        self.government_debt = (self.government_debt + policy.government_debt_change).max(0.0);
        self.government.debt = (self.government.debt + policy.government_debt_change).max(0.0);
        self.private_debt = (self.private_debt + policy.private_credit_creation).max(0.0);
        self.debt_created_this_tick =
            policy.government_debt_change + policy.private_credit_creation;
        self.wealth_shock_this_tick = policy.wealth_shock;
        self.money_created_this_tick = policy.liquidity_injection + policy.private_credit_creation;
        self.money_created_total += self.money_created_this_tick;
        if self.money_created_this_tick != 0.0 {
            self.money_events.push(MoneyEvent {
                tick: self.tick,
                kind: if self.money_created_this_tick > 0.0 {
                    "money_creation".into()
                } else {
                    "money_destruction".into()
                },
                amount: self.money_created_this_tick,
                reason: policy.description.clone(),
            });
        }
        self.last_policy_apply_tick = policy.apply_tick;
        self.recalculate_metrics();
        let after_policy = self.metric_frame();
        if let Some(event) = self
            .risk_events
            .iter_mut()
            .find(|event| event.policy_tick == policy.apply_tick)
        {
            event.after_policy = Some(after_policy);
        }
    }

    pub fn apply_next_policy(&mut self) -> bool {
        self.detect_upcoming_risk();
        let next = self
            .config
            .monetary_history
            .iter()
            .find(|policy| {
                policy.apply_tick > self.last_policy_apply_tick
                    && (!policy.optional || self.config.catastrophes)
            })
            .cloned();
        let Some(policy) = next else {
            return false;
        };
        self.apply_policy_event(&policy);
        self.pending_policy = Some(policy);
        self.recalculate_metrics();
        true
    }

    fn record_transaction(
        &mut self,
        from: u32,
        to: u32,
        kind: &str,
        amount: f64,
        region: u16,
        reason: &str,
    ) {
        if amount <= 0.0 {
            return;
        }
        self.transactions.push(Transaction {
            id: self.next_transaction_id,
            tick: self.tick,
            from,
            to,
            kind: kind.into(),
            amount,
            region,
            reason: reason.into(),
        });
        self.next_transaction_id += 1;
    }

    fn begin_institution_tick(&mut self) {
        self.transactions.clear();
        for market in &mut self.markets {
            market.supply = 0.0;
            market.demand = 0.0;
            market.volume = 0.0;
        }
        for company in &mut self.companies {
            company.employees = 0;
            company.sales = 0.0;
            if company.active {
                company.inventory += company.production_capacity;
                if let Some(market) = self.markets.iter_mut().find(|market| {
                    market.region == company.region && market.sector == company.sector
                }) {
                    market.supply += company.production_capacity;
                }
            }
        }
        for person in &self.people {
            if person.employed && !self.companies.is_empty() {
                let company_index = person.id as usize % self.companies.len();
                self.companies[company_index].employees += 1;
            }
        }
    }

    fn settle_institution_tick(&mut self) {
        let tax_rate = self.config.policy_config.income_tax_rate.clamp(0.0, 1.0);
        let transfer = self.config.policy_config.transfer_per_person.max(0.0);
        let mut flows = Vec::with_capacity(self.people.len() * 3 + self.events.len());
        for person in &self.people {
            let company_id = if self.companies.is_empty() {
                0
            } else {
                3_000_000 + person.id % self.companies.len() as u32
            };
            if person.income > 0.0 && person.employed {
                flows.push((
                    company_id,
                    person.id,
                    "wage",
                    person.income,
                    person.region,
                    "nómina por trabajo realizado",
                ));
            }
            let tax = person.income * tax_rate;
            if tax > 0.0 {
                flows.push((
                    person.id,
                    self.government.id,
                    "tax",
                    tax,
                    person.region,
                    "impuesto sobre ingresos declarados",
                ));
            }
            if transfer > 0.0 {
                flows.push((
                    self.government.id,
                    person.id,
                    "transfer",
                    transfer,
                    person.region,
                    "transferencia pública universal",
                ));
            }
        }
        for event in &self.events {
            if event.event_type == "purchase" {
                let company_id = if self.companies.is_empty() {
                    0
                } else {
                    3_000_000 + event.agent_id % self.companies.len() as u32
                };
                let market_id = 1_000_000 + event.region as u32 * 1_000;
                flows.push((
                    event.agent_id,
                    company_id,
                    "purchase",
                    event.amount,
                    event.region,
                    "compra para satisfacer una necesidad",
                ));
                if let Some(market) = self
                    .markets
                    .iter_mut()
                    .find(|market| market.id == market_id)
                {
                    market.demand += event.amount;
                    market.volume += event.amount;
                    market.supply = (market.supply - event.amount).max(0.0);
                }
            }
        }
        for (from, to, kind, amount, region, reason) in flows {
            self.record_transaction(from, to, kind, amount, region, reason);
            match kind {
                "wage" => {
                    if let Some(company) =
                        self.companies.iter_mut().find(|company| company.id == from)
                    {
                        company.cash = (company.cash - amount).max(0.0);
                    }
                }
                "tax" => {
                    self.government.cash += amount;
                    self.government.tax_revenue += amount;
                }
                "transfer" => {
                    self.government.cash -= amount;
                    self.government.spending += amount;
                }
                "purchase" => {
                    if let Some(company) =
                        self.companies.iter_mut().find(|company| company.id == to)
                    {
                        company.cash += amount;
                        company.sales += amount;
                        company.inventory = (company.inventory - amount).max(0.0);
                    }
                }
                _ => {}
            }
        }
        if self.government.cash < 0.0 {
            self.government.debt += -self.government.cash;
            self.government.cash = 0.0;
        }
        self.government_debt = self.government.debt.max(self.government_debt);
        let deposits: f64 = self.people.iter().map(|person| person.money).sum();
        let bank_count = self.banks.len().max(1) as f64;
        let per_bank_loans = self.private_debt / bank_count;
        for bank in &mut self.banks {
            bank.deposits = deposits / bank_count;
            bank.reserves = bank.deposits * bank.liquidity_ratio;
            bank.loans = per_bank_loans;
        }
        self.central_bank.base_rate = self.config.policy_config.base_interest_rate;
        self.central_bank.liquidity_injected = self.config.policy_config.liquidity_injection;
        self.central_bank.reserves = deposits * 0.2;
    }

    fn detect_compliance(&mut self) {
        let mut by_entity: HashMap<u32, Vec<&Transaction>> = HashMap::new();
        for transaction in self
            .transactions
            .iter()
            .filter(|transaction| transaction.tick == self.tick)
        {
            by_entity
                .entry(transaction.from)
                .or_default()
                .push(transaction);
            by_entity
                .entry(transaction.to)
                .or_default()
                .push(transaction);
        }
        let next_id = self.compliance_alerts.len() as u64;
        let mut alerts = Vec::new();
        for person in &self.people {
            let related = by_entity.get(&person.id).cloned().unwrap_or_default();
            let inflow: f64 = related
                .iter()
                .filter(|transaction| transaction.to == person.id)
                .map(|transaction| transaction.amount)
                .sum();
            let outflow: f64 = related
                .iter()
                .filter(|transaction| transaction.from == person.id)
                .map(|transaction| transaction.amount)
                .sum();
            let loan = related
                .iter()
                .find(|transaction| transaction.kind == "loan");
            if let Some(loan) = loan {
                if loan.amount > person.income * 8.0 + 500.0 {
                    alerts.push(ComplianceAlert {
                        id: next_id + alerts.len() as u64,
                        tick: self.tick,
                        entity_id: person.id,
                        category: "aml".into(),
                        severity: ((loan.amount - person.income * 8.0) / (loan.amount + 1.0))
                            .clamp(0.0, 1.0),
                        reason:
                            "crédito recibido incompatible con el ingreso declarado del periodo"
                                .into(),
                        related_transaction_ids: related
                            .iter()
                            .map(|transaction| transaction.id)
                            .collect(),
                        status: "open".into(),
                    });
                }
            }
            let structured: Vec<&Transaction> = related
                .iter()
                .filter(|transaction| {
                    transaction.from == person.id
                        && transaction.kind == "transfer"
                        && (900.0..=1_100.0).contains(&transaction.amount)
                })
                .copied()
                .collect();
            if structured.len() >= 3 {
                alerts.push(ComplianceAlert {
                    id: next_id + alerts.len() as u64,
                    tick: self.tick,
                    entity_id: person.id,
                    category: "aml".into(),
                    severity: (structured.len() as f64 / 6.0).clamp(0.0, 1.0),
                    reason: "transferencias repetidas cerca de un umbral sintético de reporte"
                        .into(),
                    related_transaction_ids: structured
                        .iter()
                        .map(|transaction| transaction.id)
                        .collect(),
                    status: "open".into(),
                });
            }
            let circular_ids: Vec<u64> = related
                .iter()
                .filter(|outgoing| {
                    outgoing.from == person.id
                        && outgoing.kind == "transfer"
                        && related.iter().any(|incoming| {
                            incoming.to == person.id
                                && incoming.from == outgoing.to
                                && incoming.kind == "transfer"
                        })
                })
                .flat_map(|outgoing| {
                    related
                        .iter()
                        .filter(move |incoming| {
                            incoming.to == person.id
                                && incoming.from == outgoing.to
                                && incoming.kind == "transfer"
                        })
                        .map(move |incoming| [outgoing.id, incoming.id])
                })
                .flatten()
                .collect();
            if !circular_ids.is_empty() {
                alerts.push(ComplianceAlert {
                    id: next_id + alerts.len() as u64,
                    tick: self.tick,
                    entity_id: person.id,
                    category: "aml".into(),
                    severity: 0.85,
                    reason:
                        "transferencias circulares entre intermediarios sin actividad comercial"
                            .into(),
                    related_transaction_ids: circular_ids,
                    status: "open".into(),
                });
            }
            let regions: HashSet<u16> = related
                .iter()
                .filter(|transaction| transaction.from == person.id)
                .map(|transaction| transaction.region)
                .collect();
            let cross_region_volume: f64 = related
                .iter()
                .filter(|transaction| transaction.from == person.id)
                .map(|transaction| transaction.amount)
                .sum();
            if regions.len() >= 2 && cross_region_volume > 3_000.0 {
                alerts.push(ComplianceAlert {
                    id: next_id + alerts.len() as u64,
                    tick: self.tick,
                    entity_id: person.id,
                    category: "aml".into(),
                    severity: (cross_region_volume / 10_000.0).clamp(0.0, 1.0),
                    reason:
                        "volumen transferido entre regiones incompatible con la actividad observada"
                            .into(),
                    related_transaction_ids: related
                        .iter()
                        .filter(|transaction| transaction.from == person.id)
                        .map(|transaction| transaction.id)
                        .collect(),
                    status: "open".into(),
                });
            }
            if outflow > person.income + inflow * 0.35 + 1_000.0 && person.wealth < outflow * 0.5 {
                alerts.push(ComplianceAlert {
                    id: next_id + alerts.len() as u64,
                    tick: self.tick,
                    entity_id: person.id,
                    category: "tax_evasion".into(),
                    severity: ((outflow - person.income) / (outflow + 1.0)).clamp(0.0, 1.0),
                    reason: "flujo de salida incompatible con ingreso, patrimonio y transferencias observadas".into(),
                    related_transaction_ids: related.iter().map(|transaction| transaction.id).collect(),
                    status: "open".into(),
                });
            }
        }
        for company in &self.companies {
            let related = by_entity.get(&company.id).cloned().unwrap_or_default();
            let sales: f64 = related
                .iter()
                .filter(|transaction| {
                    transaction.to == company.id && transaction.kind == "purchase"
                })
                .map(|transaction| transaction.amount)
                .sum();
            if sales > 1_000.0 && company.cash < sales * 0.1 {
                alerts.push(ComplianceAlert {
                    id: next_id + alerts.len() as u64,
                    tick: self.tick,
                    entity_id: company.id,
                    category: "tax_evasion".into(),
                    severity: (1.0 - company.cash / sales).clamp(0.0, 1.0),
                    reason: "ventas observadas altas frente a caja declarada y utilidad disponible"
                        .into(),
                    related_transaction_ids: related
                        .iter()
                        .map(|transaction| transaction.id)
                        .collect(),
                    status: "open".into(),
                });
            }
        }
        if !alerts.is_empty() {
            self.systemic_risk = (self.systemic_risk + alerts.len() as f64 * 0.01).min(1.0);
        }
        self.compliance_alerts.extend(alerts);
        if self.compliance_alerts.len() > 500 {
            let excess = self.compliance_alerts.len() - 500;
            self.compliance_alerts.drain(..excess);
        }
    }

    fn process_credit_cycle(&mut self, price: f64) {
        if self.banks.is_empty() {
            return;
        }
        let mut available_credit: f64 = self
            .banks
            .iter()
            .map(|bank| (bank.reserves * 4.0 - bank.loans).max(0.0))
            .sum();
        let minimum_loan = (price * 4.0).max(50.0);
        for index in 0..self.people.len() {
            if available_credit < minimum_loan
                || self.people[index].money >= price * 2.0
                || self.people[index].need < 0.8
            {
                continue;
            }
            let amount = minimum_loan.min(available_credit);
            let bank_index = index % self.banks.len();
            let bank_id = self.banks[bank_index].id;
            self.people[index].money += amount;
            self.people[index].debt += amount;
            self.private_debt += amount;
            self.debt_created_this_tick += amount;
            self.money_created_this_tick += amount;
            self.money_created_total += amount;
            self.banks[bank_index].loans += amount;
            available_credit -= amount;
            self.record_transaction(
                bank_id,
                self.people[index].id,
                "loan",
                amount,
                self.people[index].region,
                "crédito de liquidez para necesidad básica",
            );
        }
        for index in 0..self.companies.len() {
            if available_credit < minimum_loan || !self.companies[index].active {
                continue;
            }
            let required =
                self.companies[index].employees as f64 * self.config.scenario_config.wage;
            if self.companies[index].cash >= required * 0.75 {
                continue;
            }
            let amount = (required - self.companies[index].cash)
                .max(minimum_loan)
                .min(available_credit);
            let bank_index = index % self.banks.len();
            let bank_id = self.banks[bank_index].id;
            self.companies[index].cash += amount;
            self.companies[index].debt += amount;
            self.private_debt += amount;
            self.debt_created_this_tick += amount;
            self.money_created_this_tick += amount;
            self.money_created_total += amount;
            self.banks[bank_index].loans += amount;
            available_credit -= amount;
            self.record_transaction(
                bank_id,
                self.companies[index].id,
                "loan",
                amount,
                self.companies[index].region,
                "crédito de capital de trabajo",
            );
        }
        for index in 0..self.people.len() {
            if self.people[index].debt <= 0.0 || self.people[index].money <= price * 6.0 {
                continue;
            }
            let amount = (self.people[index].debt * 0.02)
                .min(self.people[index].money * 0.05)
                .max(0.0);
            let bank_index = index % self.banks.len();
            self.people[index].money -= amount;
            self.people[index].debt = (self.people[index].debt - amount).max(0.0);
            self.private_debt = (self.private_debt - amount).max(0.0);
            self.banks[bank_index].loans = (self.banks[bank_index].loans - amount).max(0.0);
            self.record_transaction(
                self.people[index].id,
                self.banks[bank_index].id,
                "repayment",
                amount,
                self.people[index].region,
                "amortización de crédito personal",
            );
        }
        for index in 0..self.companies.len() {
            if !self.companies[index].active
                || self.companies[index].debt <= 0.0
                || self.companies[index].cash <= minimum_loan
            {
                continue;
            }
            let amount = (self.companies[index].debt * 0.03).min(self.companies[index].cash * 0.04);
            let company_id = self.companies[index].id;
            let region = self.companies[index].region;
            self.companies[index].cash -= amount;
            self.companies[index].debt = (self.companies[index].debt - amount).max(0.0);
            self.private_debt = (self.private_debt - amount).max(0.0);
            let bank_index = index % self.banks.len();
            self.banks[bank_index].loans = (self.banks[bank_index].loans - amount).max(0.0);
            self.record_transaction(
                company_id,
                self.banks[index % self.banks.len()].id,
                "repayment",
                amount,
                region,
                "amortización de crédito empresarial",
            );
        }
        for index in 0..self.companies.len() {
            if self.companies[index].active
                && self.companies[index].debt > self.companies[index].cash * 2.0
                && self.companies[index].cash < minimum_loan
            {
                let company_id = self.companies[index].id;
                let region = self.companies[index].region;
                let loss = self.companies[index].debt;
                self.companies[index].active = false;
                self.companies[index].inventory = 0.0;
                self.companies[index].debt = 0.0;
                self.private_debt = (self.private_debt - loss).max(0.0);
                let bank_index = index % self.banks.len();
                self.banks[bank_index].loans = (self.banks[bank_index].loans - loss).max(0.0);
                self.events.push(EconomicEvent {
                    tick: self.tick,
                    event_type: "bankruptcy".into(),
                    agent_id: company_id,
                    region,
                    amount: loss,
                    reason: "insolvencia: deuda superior al doble de la caja".into(),
                });
                self.record_transaction(
                    company_id,
                    self.government.id,
                    "bankruptcy",
                    loss,
                    region,
                    "liquidación de empresa insolvente",
                );
            }
        }
        if self.money_created_this_tick > 0.0 {
            self.money_events.push(MoneyEvent {
                tick: self.tick,
                kind: "credit_creation".into(),
                amount: self.money_created_this_tick,
                reason: "crédito bancario endógeno del ciclo institucional".into(),
            });
        }
    }

    fn apply_external_shock(&mut self, rng: &mut ChaCha8Rng) {
        let probability = self
            .config
            .scenario_config
            .shock_probability
            .clamp(0.0, 1.0);
        if !self.config.catastrophes || probability == 0.0 || !rng.random_bool(probability) {
            return;
        }
        let region = rng.random_range(0..self.config.regions.max(1)) as u16;
        let (kind, reason, direction) = match rng.random_range(0..4) {
            0 => (
                "shock_oferta",
                "interrupción regional de producción y suministros",
                -1.0,
            ),
            1 => (
                "corrida_bancaria",
                "pérdida repentina de confianza en intermediarios",
                -1.0,
            ),
            2 => ("burbuja_activos", "revaluación rápida de activos", 1.0),
            _ => (
                "crisis_sanitaria",
                "caída temporal de actividad y empleo",
                -1.0,
            ),
        };
        let intensity =
            (self.config.scenario_config.shock_intensity * rng.random_range(0.7..1.3) * direction)
                .clamp(-0.8, 0.8);
        self.wealth_shock_this_tick = (self.wealth_shock_this_tick + intensity).clamp(-0.8, 0.8);
        let shock = ExternalShock {
            id: self.external_shocks.len() as u64,
            tick: self.tick,
            kind: kind.into(),
            region,
            intensity,
            reason: reason.into(),
        };
        self.events.push(EconomicEvent {
            tick: self.tick,
            event_type: "external_shock".into(),
            agent_id: self.government.id,
            region,
            amount: intensity.abs(),
            reason: reason.into(),
        });
        self.external_shocks.push(shock);
        if self.external_shocks.len() > 500 {
            let excess = self.external_shocks.len() - 500;
            self.external_shocks.drain(..excess);
        }
    }

    fn evolve_strategies(&mut self, rng: &mut ChaCha8Rng) {
        if !self.config.scenario_config.evolution {
            return;
        }
        let mutation_rate = self
            .config
            .scenario_config
            .strategy_mutation_rate
            .clamp(0.0, 1.0);
        for (index, person) in self.people.iter_mut().enumerate() {
            let previous = self
                .previous_wealth
                .get(index)
                .copied()
                .unwrap_or(person.wealth);
            if person.wealth < previous * 0.98 {
                person.strategy = Strategy::Saver;
            } else if person.income > self.config.scenario_config.wage * 0.7 && person.need < 0.5 {
                person.strategy = Strategy::Entrepreneur;
            } else if person.need > 0.78 {
                person.strategy = Strategy::Consumer;
            }
            if rng.random_bool(mutation_rate) {
                person.strategy = match rng.random_range(0..3) {
                    0 => Strategy::Saver,
                    1 => Strategy::Consumer,
                    _ => Strategy::Entrepreneur,
                };
            }
        }
    }

    pub fn step(&mut self) -> Snapshot {
        if self.tick >= self.config.ticks {
            return self.snapshot();
        }
        self.previous_wealth = self.people.iter().map(|p| p.wealth).collect();
        self.money_created_this_tick = 0.0;
        self.debt_created_this_tick = 0.0;
        self.wealth_shock_this_tick = 0.0;
        self.events.clear();
        self.detect_upcoming_risk();
        self.begin_institution_tick();
        let mut scheduled_policy = self.pending_policy.take();
        if let Some(policy) = scheduled_policy.as_ref() {
            self.debt_created_this_tick =
                policy.government_debt_change + policy.private_credit_creation;
            self.money_created_this_tick =
                policy.liquidity_injection + policy.private_credit_creation;
            self.wealth_shock_this_tick = policy.wealth_shock;
        }
        if scheduled_policy.is_none() && matches!(self.config.policy_mode, PolicyMode::Automatic) {
            let next = self
                .config
                .monetary_history
                .iter()
                .find(|policy| {
                    policy.apply_tick == self.tick
                        && policy.apply_tick > self.last_policy_apply_tick
                        && (!policy.optional || self.config.catastrophes)
                })
                .cloned();
            if let Some(policy) = next {
                self.apply_policy_event(&policy);
                scheduled_policy = Some(policy);
            }
        }
        let seed = self.config.seed ^ self.tick.wrapping_mul(0x9E3779B97F4A7C15);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        self.apply_external_shock(&mut rng);
        let wage = self.config.scenario_config.wage;
        let policy_rate = self
            .config
            .policy_config
            .base_interest_rate
            .clamp(-0.05, 1.0);
        let price = self.config.scenario_config.food_price
            * (1.0 + (self.tick as f64 / 100_000.0) + policy_rate * 0.5);
        let tax_rate = self.config.policy_config.income_tax_rate.clamp(0.0, 1.0);
        let transfer = self.config.policy_config.transfer_per_person.max(0.0);
        let scheduled_liquidity = scheduled_policy
            .as_ref()
            .map(|policy| policy.liquidity_injection)
            .unwrap_or(0.0);
        let per_person_liquidity = scheduled_liquidity / self.people.len().max(1) as f64;
        let per_person_credit = scheduled_policy
            .as_ref()
            .map(|policy| policy.private_credit_creation)
            .unwrap_or(0.0)
            / self.people.len().max(1) as f64;
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
            person.money += per_person_credit;
            person.debt = (person.debt + per_person_credit).max(0.0);
            if self.wealth_shock_this_tick != 0.0 {
                person.money = (person.money * (1.0 + self.wealth_shock_this_tick)).max(0.0);
            }
            person.need = (person.need + 0.08).min(1.0);
            let purchase_need = match person.strategy {
                Strategy::Saver => 0.58,
                Strategy::Consumer => 0.35,
                Strategy::Entrepreneur => 0.48,
            };
            if person.money >= price && person.need > purchase_need {
                person.money -= price;
                let consumption = match person.strategy {
                    Strategy::Saver => 0.55,
                    Strategy::Consumer => 0.72,
                    Strategy::Entrepreneur => 0.62,
                };
                person.need = (person.need - consumption).max(0.0);
                self.events.push(EconomicEvent {
                    tick: self.tick,
                    event_type: "purchase".into(),
                    agent_id: person.id,
                    region: person.region,
                    amount: price,
                    reason: "basic need and liquidity".into(),
                });
            }
            person.wealth = (person.money - person.debt).max(0.0);
            person.risk_score = ((person.income * 12.0 + person.money) / (person.wealth + 1.0)
                - 1.0)
                .clamp(0.0, 1.0);
            if self.wealth_shock_this_tick < -0.2 && rng.random_bool(0.15) {
                person.employed = false;
            } else if self.wealth_shock_this_tick > 0.1 && rng.random_bool(0.08) {
                person.employed = true;
            } else if rng.random_bool(0.01) {
                person.employed = !person.employed;
            }
        }
        self.evolve_strategies(&mut rng);
        self.process_credit_cycle(price);
        self.settle_institution_tick();
        self.detect_compliance();
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
        let mut regional_totals = vec![(0.0, 0_u64); self.config.regions.max(1)];
        for person in &self.people {
            let region = person.region as usize;
            if let Some((regional_total, regional_count)) = regional_totals.get_mut(region) {
                *regional_total += person.wealth;
                *regional_count += 1;
            }
        }
        let regional_means: Vec<f64> = regional_totals
            .iter()
            .filter_map(|(regional_total, regional_count)| {
                (*regional_count > 0).then_some(*regional_total / *regional_count as f64)
            })
            .collect();
        let regional_wealth_gap = regional_means
            .iter()
            .copied()
            .reduce(f64::max)
            .zip(regional_means.iter().copied().reduce(f64::min))
            .map(|(maximum, minimum)| ((maximum - minimum) / total.max(1.0)).clamp(0.0, 1.0))
            .unwrap_or(0.0);
        let bank_loans: f64 = self.banks.iter().map(|bank| bank.loans).sum();
        let bank_concentration = self
            .banks
            .iter()
            .map(|bank| bank.loans)
            .reduce(f64::max)
            .map(|largest| {
                if bank_loans > 0.0 {
                    largest / bank_loans
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        let bankruptcies = self
            .companies
            .iter()
            .filter(|company| !company.active)
            .count() as u64;
        let social_mobility =
            if self.previous_wealth.len() == self.people.len() && !self.people.is_empty() {
                let mut previous_order: Vec<usize> = (0..self.people.len()).collect();
                previous_order.sort_by(|left, right| {
                    self.previous_wealth[*left].total_cmp(&self.previous_wealth[*right])
                });
                let mut current_order: Vec<usize> = (0..self.people.len()).collect();
                current_order.sort_by(|left, right| {
                    self.people[*left]
                        .wealth
                        .total_cmp(&self.people[*right].wealth)
                });
                let mut previous_deciles = vec![0_u8; self.people.len()];
                let mut current_deciles = vec![0_u8; self.people.len()];
                for (rank, index) in previous_order.into_iter().enumerate() {
                    previous_deciles[index] = ((rank * 10) / self.people.len()) as u8;
                }
                for (rank, index) in current_order.into_iter().enumerate() {
                    current_deciles[index] = ((rank * 10) / self.people.len()) as u8;
                }
                previous_deciles
                    .iter()
                    .zip(current_deciles.iter())
                    .filter(|(previous, current)| previous != current)
                    .count() as f64
                    / self.people.len() as f64
            } else {
                0.0
            };
        let monetary_regime = self
            .config
            .monetary_history
            .iter()
            .find(|policy| policy.apply_tick == self.last_policy_apply_tick)
            .map(|policy| policy.era.clone())
            .unwrap_or_else(|| "commodity".into());
        self.metrics = Metrics {
            gdp: income,
            gini: gini.clamp(0.0, 1.0),
            unemployment: 1.0 - employed / n,
            inflation: self.tick as f64 / 100_000.0,
            tax_revenue: income * tax_rate,
            trade_volume: self.events.iter().map(|e| e.amount).sum(),
            alerts: self.people.iter().filter(|p| p.risk_score > 0.8).count() as u64
                + self
                    .risk_events
                    .iter()
                    .filter(|event| !event.countermeasure_applied)
                    .count() as u64,
            mean_wealth: total / n,
            money_supply: self.people.iter().map(|p| p.money).sum(),
            interest_rate: self.config.policy_config.base_interest_rate,
            top_10_wealth_share: if total > 0.0 { top_wealth / total } else { 0.0 },
            money_created_this_tick: self.money_created_this_tick,
            cumulative_money_created: self.money_created_total,
            government_debt: self.government_debt,
            private_debt: self.private_debt,
            debt_created_this_tick: self.debt_created_this_tick,
            monetary_regime,
            wealth_shock: self.wealth_shock_this_tick,
            adverse_events: self.risk_events.len() as u64,
            open_risks: self
                .risk_events
                .iter()
                .filter(|event| !event.countermeasure_applied)
                .count() as u64,
            countermeasures: self
                .risk_events
                .iter()
                .filter(|event| event.countermeasure_applied)
                .count() as u64,
            systemic_risk: self.systemic_risk,
            security_level: self.security_level,
            aml_alerts: self
                .compliance_alerts
                .iter()
                .filter(|alert| alert.category == "aml" && alert.status == "open")
                .count() as u64,
            tax_alerts: self
                .compliance_alerts
                .iter()
                .filter(|alert| alert.category == "tax_evasion" && alert.status == "open")
                .count() as u64,
            bankruptcies,
            bank_concentration,
            regional_wealth_gap,
            social_mobility,
        };
    }

    pub fn snapshot(&self) -> Snapshot {
        let columns = (self.people.len() as f32).sqrt().ceil().max(1.0) as u32;
        let rows = ((self.people.len() as f32 / columns as f32).ceil().max(1.0)) as u32;
        let mut agents: Vec<VisualAgent> = self
            .people
            .iter()
            .map(|p| VisualAgent {
                id: p.id,
                kind: "person".into(),
                region: p.region,
                sector: 0,
                x: ((p.id % columns) as f32 + 0.5) / columns as f32,
                y: ((p.id / columns) as f32 + 0.5) / rows as f32,
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
                sector: 0,
                x: 0.1 + region as f32 * 0.8 / self.config.regions.max(1) as f32,
                y: 0.88,
                size: 12.0,
                risk_score: 0.0,
                employed: true,
                wealth: self.metrics.money_supply as f32,
                wealth_delta: 0.0,
            });
        }
        for (index, company) in self.companies.iter().enumerate() {
            agents.push(VisualAgent {
                id: company.id,
                kind: "company".into(),
                region: company.region,
                sector: company.sector,
                x: 0.1
                    + ((index % self.config.regions.max(1)) as f32 + 0.35) * 0.8
                        / self.config.regions.max(1) as f32,
                y: 0.18 + (index % 5) as f32 * 0.07,
                size: (company.cash.max(1.0).sqrt() as f32 / 18.0).clamp(5.0, 16.0),
                risk_score: if company.active { 0.0 } else { 1.0 },
                employed: company.active,
                wealth: company.cash as f32,
                wealth_delta: company.sales as f32,
            });
        }
        for (index, bank) in self.banks.iter().enumerate() {
            agents.push(VisualAgent {
                id: bank.id,
                kind: "bank".into(),
                region: bank.region,
                sector: 0,
                x: 0.12
                    + ((index % self.config.regions.max(1)) as f32 + 0.65) * 0.8
                        / self.config.regions.max(1) as f32,
                y: 0.72 + (index % 3) as f32 * 0.06,
                size: (bank.deposits.max(1.0).sqrt() as f32 / 25.0).clamp(7.0, 18.0),
                risk_score: (1.0 - bank.liquidity_ratio).clamp(0.0, 1.0) as f32,
                employed: true,
                wealth: bank.deposits as f32,
                wealth_delta: bank.loans as f32,
            });
        }
        agents.push(VisualAgent {
            id: self.government.id,
            kind: "government".into(),
            region: 0,
            sector: 0,
            x: 0.08,
            y: 0.08,
            size: (self.government.debt.max(1.0).sqrt() as f32 / 35.0).clamp(8.0, 20.0),
            risk_score: (self.government.debt / (self.metrics.gdp + 1.0)).clamp(0.0, 1.0) as f32,
            employed: true,
            wealth: self.government.cash as f32,
            wealth_delta: self.government.tax_revenue as f32,
        });
        agents.push(VisualAgent {
            id: self.central_bank.id,
            kind: "central_bank".into(),
            region: 0,
            sector: 0,
            x: 0.5,
            y: 0.08,
            size: 18.0,
            risk_score: 0.0,
            employed: true,
            wealth: self.metrics.money_supply as f32,
            wealth_delta: self.metrics.money_created_this_tick as f32,
        });
        let mut edges = Vec::with_capacity(self.people.len() * 2);
        for region in 0..self.config.regions.max(1) as u16 {
            edges.push(VisualEdge {
                source: self.central_bank.id,
                target: 1_000_000 + region as u32,
                kind: "issuance".into(),
                weight: 2.5,
            });
        }
        for company in &self.companies {
            edges.push(VisualEdge {
                source: company.id,
                target: 1_000_000 + company.region as u32,
                kind: "supply".into(),
                weight: (company.sales / 100.0).sqrt().clamp(0.4, 3.0) as f32,
            });
        }
        for bank in &self.banks {
            edges.push(VisualEdge {
                source: bank.id,
                target: self.central_bank.id,
                kind: "credit".into(),
                weight: (bank.loans / 100_000.0).sqrt().clamp(0.4, 2.5) as f32,
            });
        }
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
        let active_policy = self
            .config
            .monetary_history
            .iter()
            .find(|policy| policy.apply_tick == self.last_policy_apply_tick)
            .cloned();
        let next_policy = self
            .config
            .monetary_history
            .iter()
            .find(|policy| {
                policy.apply_tick > self.last_policy_apply_tick
                    && (!policy.optional || self.config.catastrophes)
            })
            .cloned();
        Snapshot {
            tick: self.tick,
            agents,
            edges,
            metrics: self.metrics.clone(),
            policy: self.config.policy_config.clone(),
            monetary_history: self.config.monetary_history.clone(),
            money_events: self.money_events.clone(),
            active_policy,
            next_policy,
            policy_mode: self.config.policy_mode.clone(),
            risk_events: self.risk_events.clone(),
            companies: self.companies.clone(),
            banks: self.banks.clone(),
            government: self.government.clone(),
            central_bank: self.central_bank.clone(),
            markets: self.markets.clone(),
            transactions: self
                .transactions
                .iter()
                .rev()
                .take(2_500)
                .cloned()
                .collect(),
            compliance_alerts: self
                .compliance_alerts
                .iter()
                .rev()
                .take(100)
                .cloned()
                .collect(),
            external_shocks: self
                .external_shocks
                .iter()
                .rev()
                .take(100)
                .cloned()
                .collect(),
        }
    }

    pub fn apply_policy(&mut self, policy: PolicyConfig) {
        self.config.policy_config = policy;
        self.recalculate_metrics();
    }

    pub fn inject_external_shock(&mut self, kind: &str, region: u16, intensity: f64) -> bool {
        let region = region % self.config.regions.max(1) as u16;
        let intensity = intensity.clamp(0.0, 0.8);
        let (reason, direction) = match kind {
            "shock_oferta" => ("interrupción manual de producción y suministros", -1.0),
            "corrida_bancaria" => ("pérdida manual de confianza en intermediarios", -1.0),
            "burbuja_activos" => ("revaluación manual y acelerada de activos", 1.0),
            "crisis_sanitaria" => ("caída manual de actividad y empleo", -1.0),
            _ => return false,
        };
        let signed_intensity = intensity * direction;
        for person in &mut self.people {
            if person.region == region {
                person.money = (person.money * (1.0 + signed_intensity)).max(0.0);
                person.wealth = (person.money - person.debt).max(0.0);
                if kind == "crisis_sanitaria" {
                    person.employed = false;
                }
            }
        }
        match kind {
            "shock_oferta" | "crisis_sanitaria" => {
                for company in &mut self.companies {
                    if company.region == region {
                        company.production_capacity *= 1.0 - intensity;
                    }
                }
            }
            "corrida_bancaria" => {
                for bank in &mut self.banks {
                    if bank.region == region {
                        bank.reserves *= 1.0 - intensity;
                    }
                }
            }
            "burbuja_activos" => {
                for company in &mut self.companies {
                    if company.region == region {
                        company.price *= 1.0 + intensity;
                    }
                }
            }
            _ => {}
        }
        self.wealth_shock_this_tick = signed_intensity;
        self.systemic_risk = (self.systemic_risk + intensity * 0.35).clamp(0.0, 1.0);
        self.external_shocks.push(ExternalShock {
            id: self.external_shocks.len() as u64,
            tick: self.tick,
            kind: kind.into(),
            region,
            intensity: signed_intensity,
            reason: reason.into(),
        });
        self.events.push(EconomicEvent {
            tick: self.tick,
            event_type: "external_shock".into(),
            agent_id: self.government.id,
            region,
            amount: intensity,
            reason: reason.into(),
        });
        if self.external_shocks.len() > 500 {
            let excess = self.external_shocks.len() - 500;
            self.external_shocks.drain(..excess);
        }
        self.recalculate_metrics();
        true
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
        assert_ne!(
            serde_json::to_string(&a.people).unwrap(),
            serde_json::to_string(&b.people).unwrap()
        );
    }
    #[test]
    fn gini_is_bounded() {
        let mut s = Simulation::new(config(42));
        s.step();
        assert!((0.0..=1.0).contains(&s.metrics.gini));
    }

    #[test]
    fn extended_distribution_metrics_are_bounded_and_exposed() {
        let mut config = config(42);
        config.agent_count = 32;
        config.regions = 4;
        let mut s = Simulation::new(config);
        s.step();

        assert!((0.0..=1.0).contains(&s.metrics.bank_concentration));
        assert!((0.0..=1.0).contains(&s.metrics.regional_wealth_gap));
        assert!((0.0..=1.0).contains(&s.metrics.social_mobility));
        assert_eq!(s.metrics.bankruptcies, 0);
        let encoded = serde_json::to_string(&s.metrics).unwrap();
        assert!(encoded.contains("bank_concentration"));
        assert!(encoded.contains("regional_wealth_gap"));
    }
    #[test]
    fn no_negative_money() {
        let mut s = Simulation::new(config(42));
        for _ in 0..10 {
            s.step();
        }
        assert!(s.people.iter().all(|p| p.money >= 0.0));
    }

    #[test]
    fn initial_endowment_is_equal() {
        let s = Simulation::new(config(42));
        assert!(s.people.iter().all(|p| p.wealth == 2_500.0));
        assert_eq!(s.metrics.gini, 0.0);
        assert_eq!(s.money_events[0].kind, "genesis");
    }

    #[test]
    fn economic_cycle_records_income_taxes_and_purchases() {
        let mut s = Simulation::new(config(42));
        let initial_money: f64 = s.people.iter().map(|person| person.money).sum();

        let snapshot = s.step();

        assert!(s.people.iter().any(|person| person.income > 0.0));
        assert!(s.metrics.gdp > 0.0);
        assert!(s.metrics.tax_revenue > 0.0);
        assert!(s.metrics.trade_volume > 0.0);
        assert!(s.events.iter().all(|event| event.tick == 0));
        assert_eq!(snapshot.tick, 1);
        assert!(s.people.iter().map(|person| person.money).sum::<f64>() > initial_money);
    }

    #[test]
    fn historical_schedule_is_ordered_and_explainable() {
        let history = Simulation::default_monetary_history(1_000);
        assert!(history.len() >= 60);
        assert!(history
            .windows(2)
            .all(|pair| pair[0].apply_tick < pair[1].apply_tick));
        assert!(history.iter().all(|policy| {
            !policy.name.trim().is_empty() && !policy.description.trim().is_empty()
        }));
        assert!(history
            .iter()
            .any(|policy| policy.name == "Dinero fiduciario"));
        assert!(history
            .iter()
            .any(|policy| policy.name == "Reestructuración de deuda soberana"));
    }

    #[test]
    fn automatic_mode_applies_the_last_historical_phase() {
        let mut config = config(42);
        config.ticks = 140;
        config.catastrophes = true;
        let mut s = Simulation::new(config);

        for _ in 0..137 {
            s.step();
        }

        assert_eq!(s.last_policy_apply_tick, 136);
        assert_eq!(
            s.snapshot().active_policy.unwrap().name,
            "Reestructuración de deuda soberana"
        );
        assert!(s.snapshot().monetary_history.iter().any(|policy| {
            policy.apply_tick == 136 && policy.name == "Reestructuración de deuda soberana"
        }));
    }

    #[test]
    fn manual_mode_can_apply_every_historical_phase_in_order() {
        let mut config = config(42);
        config.policy_mode = PolicyMode::Manual;
        config.catastrophes = true;
        let mut s = Simulation::new(config);
        let expected: Vec<String> = s
            .config
            .monetary_history
            .iter()
            .filter(|policy| policy.apply_tick > 0)
            .map(|policy| policy.name.clone())
            .collect();

        for name in &expected {
            assert!(s.apply_next_policy(), "could not apply phase: {name}");
            assert_eq!(s.snapshot().active_policy.unwrap().name, *name);
        }
        assert!(!s.apply_next_policy());
        assert_eq!(s.risk_events.len(), expected.len());
    }

    #[test]
    fn catastrophe_phases_are_optional_and_configurable() {
        let mut ordinary = config(42);
        ordinary.ticks = 48;
        let mut ordinary_sim = Simulation::new(ordinary);
        for _ in 0..48 {
            ordinary_sim.step();
        }
        assert!(!ordinary_sim
            .risk_events
            .iter()
            .any(|event| event.policy_tick == 44));

        let mut crisis = config(42);
        crisis.ticks = 48;
        crisis.catastrophes = true;
        let mut crisis_sim = Simulation::new(crisis);
        for _ in 0..48 {
            crisis_sim.step();
        }
        assert!(crisis_sim
            .risk_events
            .iter()
            .any(|event| event.policy_tick == 44));
    }

    #[test]
    fn reproducible_external_shocks_and_strategy_evolution() {
        let mut config = config(42);
        config.catastrophes = true;
        config.scenario_config.evolution = true;
        config.scenario_config.shock_probability = 1.0;
        config.scenario_config.shock_intensity = 0.2;
        let mut a = Simulation::new(config.clone());
        let mut b = Simulation::new(config);
        let initial = a
            .people
            .iter()
            .map(|person| person.strategy.clone())
            .collect::<Vec<_>>();

        a.step();
        b.step();

        assert!(!a.external_shocks.is_empty());
        assert_eq!(
            serde_json::to_string(&a.external_shocks).unwrap(),
            serde_json::to_string(&b.external_shocks).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&a.people).unwrap(),
            serde_json::to_string(&b.people).unwrap()
        );
        assert_ne!(
            initial,
            a.people
                .iter()
                .map(|person| person.strategy.clone())
                .collect::<Vec<_>>()
        );
        assert!(a
            .events
            .iter()
            .any(|event| event.event_type == "external_shock"));
    }

    #[test]
    fn manual_external_shock_changes_region_and_records_cause() {
        let mut s = Simulation::new(config(42));
        let before_capacity = s.companies[0].production_capacity;
        let before_wealth: f64 = s
            .people
            .iter()
            .filter(|person| person.region == 0)
            .map(|person| person.wealth)
            .sum();

        assert!(s.inject_external_shock("shock_oferta", 0, 0.25));

        assert!(s.companies[0].production_capacity < before_capacity);
        let after_wealth: f64 = s
            .people
            .iter()
            .filter(|person| person.region == 0)
            .map(|person| person.wealth)
            .sum();
        assert!(after_wealth < before_wealth);
        assert_eq!(s.external_shocks.len(), 1);
        assert_eq!(s.external_shocks[0].kind, "shock_oferta");
        assert!(s
            .events
            .iter()
            .any(|event| event.event_type == "external_shock"));
        assert!(!s.inject_external_shock("unknown", 0, 0.2));
    }

    #[test]
    fn policy_override_changes_live_parameters_without_resetting_agents() {
        let mut s = Simulation::new(config(42));
        s.step();
        let people_before = s.people.len();
        s.apply_policy(PolicyConfig {
            income_tax_rate: 0.35,
            transfer_per_person: 45.0,
            base_interest_rate: 0.09,
            liquidity_injection: 12_000.0,
        });

        assert_eq!(s.people.len(), people_before);
        assert_eq!(s.metrics.interest_rate, 0.09);
        assert_eq!(s.config.policy_config.income_tax_rate, 0.35);
        assert_eq!(s.config.policy_config.liquidity_injection, 12_000.0);
    }

    #[test]
    fn fiscal_policy_changes_measurable_outcomes_without_changing_seeded_agents() {
        let mut baseline = Simulation::new(config(42));
        let mut redistributive = Simulation::new(config(42));
        redistributive.apply_policy(PolicyConfig {
            income_tax_rate: 0.45,
            transfer_per_person: 120.0,
            base_interest_rate: 0.04,
            liquidity_injection: 0.0,
        });

        baseline.step();
        redistributive.step();

        assert_eq!(baseline.people.len(), redistributive.people.len());
        assert_eq!(baseline.metrics.gdp, redistributive.metrics.gdp);
        assert!(redistributive.metrics.tax_revenue > baseline.metrics.tax_revenue);
        let baseline_wealth: f64 = baseline.people.iter().map(|person| person.wealth).sum();
        let redistributive_wealth: f64 = redistributive
            .people
            .iter()
            .map(|person| person.wealth)
            .sum();
        assert_ne!(baseline_wealth, redistributive_wealth);
        assert!(redistributive
            .transactions
            .iter()
            .any(|transaction| transaction.kind == "transfer"));
    }

    #[test]
    fn step_after_horizon_returns_the_same_snapshot() {
        let mut config = config(42);
        config.ticks = 1;
        let mut s = Simulation::new(config);
        s.step();
        let first = serde_json::to_string(&s.snapshot()).unwrap();
        let second = serde_json::to_string(&s.step()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn snapshot_contains_visual_nodes_and_interactions() {
        let mut s = Simulation::new(config(42));
        s.step();
        let snapshot = s.snapshot();
        assert_eq!(
            snapshot.agents.len(),
            200 + 4 + s.companies.len() + s.banks.len() + 2
        );
        assert!(snapshot.agents.iter().any(|agent| agent.kind == "market"));
        assert!(snapshot
            .agents
            .iter()
            .any(|agent| agent.kind == "central_bank"));
        assert!(!snapshot.edges.is_empty());
    }

    #[test]
    fn institutional_cycle_produces_balances_and_transactions() {
        let mut s = Simulation::new(config(42));
        s.step();

        assert!(!s.companies.is_empty());
        assert!(s.companies.iter().all(|company| company.inventory >= 0.0));
        assert!(!s.banks.is_empty());
        assert!(s.banks.iter().all(|bank| bank.deposits >= bank.reserves));
        assert!(s.transactions.iter().any(|tx| tx.kind == "wage"));
        assert!(s.transactions.iter().any(|tx| tx.kind == "tax"));
        assert!(s.transactions.iter().any(|tx| tx.kind == "purchase"));
        assert!(s
            .transactions
            .iter()
            .all(|tx| tx.tick == 0 && tx.amount > 0.0));
        let ids: std::collections::HashSet<u64> = s.transactions.iter().map(|tx| tx.id).collect();
        assert_eq!(ids.len(), s.transactions.len());
    }

    #[test]
    fn credit_cycle_creates_traceable_loan_and_debt() {
        let mut config = config(42);
        config.scenario_config.food_price = 1_000.0;
        let mut s = Simulation::new(config);
        s.people[0].money = 0.0;
        s.people[0].need = 1.0;
        s.people[0].employed = false;

        s.step();

        assert!(s.transactions.iter().any(|tx| tx.kind == "loan"));
        assert!(s.people.iter().any(|person| person.debt > 0.0));
        assert!(s.private_debt > 0.0);
        assert!(s
            .money_events
            .iter()
            .any(|event| event.kind == "credit_creation"));
        assert!(s
            .compliance_alerts
            .iter()
            .any(|alert| alert.category == "aml" && alert.entity_id == 0));
    }

    #[test]
    fn bank_exposure_reconciles_repayments_and_company_default() {
        let mut s = Simulation::new(config(42));
        s.people[0].debt = 100.0;
        s.people[0].money = 1_000.0;
        s.people[0].need = 0.0;
        s.banks[0].loans = 100.0;

        s.process_credit_cycle(12.0);

        assert!(s.people[0].debt < 100.0);
        assert!(s.banks[0].loans < 100.0);

        let company = &mut s.companies[0];
        company.cash = 0.0;
        company.debt = 1_000.0;
        s.banks[0].loans += 1_000.0;

        s.process_credit_cycle(12.0);

        assert!(!s.companies[0].active);
        assert!(s.banks[0].loans < 1_000.0);
        assert!(s.banks.iter().all(|bank| bank.loans >= 0.0));
    }

    #[test]
    fn tax_alert_explains_unmatched_outflow() {
        let mut s = Simulation::new(config(42));
        s.people[0].money = 0.0;
        s.people[0].wealth = 0.0;
        s.transactions.push(Transaction {
            id: 77,
            tick: 0,
            from: 0,
            to: 3_000_000,
            kind: "purchase".into(),
            amount: 2_500.0,
            region: s.people[0].region,
            reason: "flujo sintético de prueba".into(),
        });

        s.detect_compliance();

        let alert = s
            .compliance_alerts
            .iter()
            .find(|alert| alert.category == "tax_evasion")
            .expect("tax mismatch should create an alert");
        assert_eq!(alert.entity_id, 0);
        assert_eq!(alert.related_transaction_ids, vec![77]);
        assert!(alert.reason.contains("flujo"));
    }

    #[test]
    fn aml_rules_explain_structuring_circularity_and_cross_region_flow() {
        let mut s = Simulation::new(config(42));
        for (id, region) in [(77, 0), (78, 1), (79, 2)] {
            s.transactions.push(Transaction {
                id,
                tick: 0,
                from: 0,
                to: 9,
                kind: "transfer".into(),
                amount: 1_001.0,
                region,
                reason: "transferencia sintética de prueba".into(),
            });
        }
        for (id, amount) in [(80, 300.0), (81, 250.0), (82, 200.0)] {
            s.transactions.push(Transaction {
                id,
                tick: 0,
                from: 9,
                to: 0,
                kind: "transfer".into(),
                amount,
                region: 0,
                reason: "retorno sintético de prueba".into(),
            });
        }

        s.detect_compliance();

        let alerts: Vec<&ComplianceAlert> = s
            .compliance_alerts
            .iter()
            .filter(|alert| alert.entity_id == 0 && alert.category == "aml")
            .collect();
        assert!(alerts.iter().any(|alert| alert.reason.contains("umbral")));
        assert!(alerts
            .iter()
            .any(|alert| alert.reason.contains("circulares")));
        assert!(alerts.iter().any(|alert| alert.reason.contains("regiones")));
        assert!(alerts
            .iter()
            .all(|alert| !alert.related_transaction_ids.is_empty()));
    }

    #[test]
    fn ten_thousand_agents_complete_a_tick_with_bounded_visual_payload() {
        let mut config = config(42);
        config.agent_count = 10_000;
        config.ticks = 1;
        let mut s = Simulation::new(config);

        let snapshot = s.step();

        assert_eq!(s.people.len(), 10_000);
        assert_eq!(snapshot.tick, 1);
        assert!(snapshot.agents.len() >= 10_000);
        assert!(snapshot.transactions.len() <= 2_500);
        assert!(s.people.iter().all(|person| person.money >= 0.0));
    }

    #[test]
    fn insolvent_company_is_liquidated_with_explainable_event() {
        let mut s = Simulation::new(config(42));
        let company = &mut s.companies[0];
        company.cash = 0.0;
        company.debt = 1_000.0;

        s.process_credit_cycle(12.0);

        assert!(!s.companies[0].active);
        assert!(s
            .events
            .iter()
            .any(|event| event.event_type == "bankruptcy"));
        assert!(s
            .transactions
            .iter()
            .any(|transaction| transaction.kind == "bankruptcy"));
    }

    #[test]
    fn scheduled_policy_creates_money_at_tick() {
        let mut config = config(42);
        config.ticks = 20;
        let mut s = Simulation::new(config);
        for _ in 0..13 {
            s.step();
        }
        assert_eq!(s.tick, 13);
        assert_eq!(s.metrics.money_created_this_tick, 650_000.0);
        assert!(s
            .money_events
            .iter()
            .any(|event| event.kind == "money_creation"));
        assert!(s.metrics.cumulative_money_created > s.people.len() as f64 * 2_500.0);
    }

    #[test]
    fn manual_mode_evolves_until_next_policy_is_requested() {
        let mut config = config(42);
        config.ticks = 10;
        config.policy_mode = PolicyMode::Manual;
        let mut s = Simulation::new(config);
        s.step();
        s.step();
        assert_eq!(s.tick, 2);
        assert_eq!(s.last_policy_apply_tick, 0);
        assert_eq!(s.metrics.debt_created_this_tick, 0.0);
        assert!(s.apply_next_policy());
        assert_eq!(s.tick, 2);
        assert_eq!(s.last_policy_apply_tick, 2);
        assert_eq!(
            s.snapshot().active_policy.unwrap().name,
            "Trueque y unidad de cuenta"
        );
        assert!(s.risk_events[0].after_policy.is_some());
    }

    #[test]
    fn detects_risk_and_measures_countermeasure() {
        let mut config = config(42);
        config.policy_mode = PolicyMode::Manual;
        let mut s = Simulation::new(config);
        s.step();
        assert_eq!(s.risk_events.len(), 1);
        assert_eq!(s.metrics.open_risks, 1);
        let before = s.risk_events[0].before.systemic_risk;
        assert!(s.apply_countermeasure());
        assert_eq!(s.metrics.open_risks, 0);
        assert_eq!(s.metrics.countermeasures, 1);
        assert!(
            s.risk_events[0]
                .after_countermeasure
                .as_ref()
                .unwrap()
                .systemic_risk
                < before
        );
    }

    #[test]
    fn risk_profiles_cover_the_main_adverse_systems() {
        let policies = Simulation::default_monetary_history(1_000);
        let categories: std::collections::HashSet<String> = policies
            .iter()
            .map(|policy| Simulation::risk_profile(policy).0)
            .collect();

        for category in [
            "currency_security",
            "bank_run",
            "reserve_drain",
            "supply_inflation",
            "asset_bubble",
            "exchange_attack",
            "sovereign_stress",
            "liquidity_freeze",
            "payment_integrity",
        ] {
            assert!(
                categories.contains(category),
                "missing risk category: {category}"
            );
        }
    }

    #[test]
    fn currency_countermeasure_improves_security_and_closes_alert() {
        let mut custom = config(42);
        custom.policy_mode = PolicyMode::Manual;
        custom.monetary_history = vec![MonetaryPolicy {
            apply_tick: 2,
            era: "convertible".into(),
            name: "Papel convertible".into(),
            description: "Recibos respaldados por reservas".into(),
            base_interest_rate: 0.04,
            liquidity_injection: 0.0,
            government_debt_change: 0.0,
            private_credit_creation: 0.0,
            wealth_shock: 0.0,
            catastrophe: false,
            optional: false,
        }];
        let mut s = Simulation::new(custom);
        s.step();
        let before = s.security_level;
        assert_eq!(s.risk_events[0].category, "currency_security");
        assert!(s.apply_countermeasure());
        assert_eq!(s.metrics.open_risks, 0);
        assert!(s.security_level > before);
    }

    #[test]
    fn risk_countermeasure_is_idempotent_when_no_alert_is_open() {
        let mut s = Simulation::new(config(42));
        assert!(!s.apply_countermeasure());
        s.step();
        assert!(s.apply_countermeasure());
        assert!(!s.apply_countermeasure());
    }

    #[test]
    fn debt_and_money_metrics_match_policy_creation() {
        let mut custom = config(42);
        custom.policy_mode = PolicyMode::Manual;
        custom.monetary_history = vec![MonetaryPolicy {
            apply_tick: 2,
            era: "credit".into(),
            name: "Expansión de crédito".into(),
            description: "El crédito crea depósitos".into(),
            base_interest_rate: 0.04,
            liquidity_injection: 1_000.0,
            government_debt_change: 2_000.0,
            private_credit_creation: 3_000.0,
            wealth_shock: 0.0,
            catastrophe: false,
            optional: false,
        }];
        let mut s = Simulation::new(custom);
        s.step();
        assert!(s.apply_next_policy());
        assert!(s.metrics.government_debt >= 2_000.0);
        assert_eq!(s.metrics.private_debt, 3_000.0);
        assert_eq!(s.metrics.money_created_this_tick, 4_000.0);
        assert_eq!(s.money_events.last().unwrap().amount, 4_000.0);
    }

    #[test]
    fn config_hash_is_stable_and_sensitive_to_configuration() {
        let mut first = Simulation::new(config(42));
        let second = Simulation::new(config(42));
        assert_eq!(first.config_hash(), second.config_hash());
        first.config.policy_config.base_interest_rate = 0.08;
        assert_ne!(first.config_hash(), second.config_hash());
    }

    #[test]
    fn long_history_keeps_risk_as_a_live_signal() {
        let mut config = config(42);
        config.ticks = 160;
        config.catastrophes = true;
        let mut s = Simulation::new(config);
        for _ in 0..140 {
            s.step();
        }
        assert_eq!(s.tick, 140);
        assert!(s.metrics.adverse_events >= 60);
        assert!(s.metrics.systemic_risk < 0.9);
        let before = s.metrics.systemic_risk;
        assert!(s.apply_countermeasure());
        assert!(s.metrics.systemic_risk < before);
    }
}
