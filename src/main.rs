use arrow_array::{ArrayRef, Float64Array, RecordBatch, UInt64Array};
use arrow_schema::{DataType, Field, Schema};
use axum::{
    body::Body,
    extract::ws::{Message, WebSocket},
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use globaleconomy::{
    EconomicEvent, PolicyConfig, PolicyMode, RunConfig, Simulation, Snapshot, Transaction,
};
use parquet::arrow::ArrowWriter;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    runs: Arc<RwLock<HashMap<Uuid, RunRecord>>>,
    storage_dir: Arc<PathBuf>,
}
#[derive(Serialize, Deserialize)]
struct RunRecord {
    sim: Simulation,
    status: String,
    created_at: DateTime<Utc>,
    history: Vec<Snapshot>,
}
#[derive(Serialize)]
struct RunCreated {
    run_id: Uuid,
    status: String,
    created_at: DateTime<Utc>,
    config_hash: String,
}
#[derive(Serialize)]
struct StatusResponse {
    run_id: Uuid,
    status: String,
    tick: u64,
}

#[derive(Serialize)]
struct RunSummary {
    run_id: Uuid,
    status: String,
    tick: u64,
    created_at: DateTime<Utc>,
    config_hash: String,
    history_ticks: Vec<u64>,
}

#[derive(Deserialize)]
struct CompareRequest {
    run_ids: Vec<Uuid>,
    tick: Option<u64>,
}

#[derive(Serialize)]
struct ComparisonRow {
    run_id: Uuid,
    tick: u64,
    metrics: globaleconomy::Metrics,
}

#[derive(Serialize)]
struct ComparisonResponse {
    requested_tick: Option<u64>,
    runs: Vec<ComparisonRow>,
}

#[derive(Serialize)]
struct MetricPoint {
    tick: u64,
    metrics: globaleconomy::Metrics,
}

#[derive(Deserialize)]
struct PersistedMetricSnapshot {
    tick: u64,
    metrics: globaleconomy::Metrics,
}

#[derive(Default, Deserialize)]
struct EventQuery {
    #[serde(rename = "type")]
    event_type: Option<String>,
    tick: Option<u64>,
    agent: Option<u32>,
    region: Option<u16>,
    risk: Option<bool>,
    min_amount: Option<f64>,
    max_amount: Option<f64>,
}

#[derive(Default, Deserialize)]
struct TransactionQuery {
    tick: Option<u64>,
    kind: Option<String>,
    agent: Option<u32>,
    region: Option<u16>,
    min_amount: Option<f64>,
    max_amount: Option<f64>,
}

const HISTORY_LIMIT: usize = 240;
const DATA_DIR_ENV: &str = "GLOBALECONOMY_DATA_DIR";
const PORT_ENV: &str = "GLOBALECONOMY_PORT";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let storage_dir =
        PathBuf::from(std::env::var(DATA_DIR_ENV).unwrap_or_else(|_| "data/runs".to_string()));
    let runs = load_runs(&storage_dir);
    let state = AppState {
        runs: Arc::new(RwLock::new(runs)),
        storage_dir: Arc::new(storage_dir),
    };
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/runs", post(create_run))
        .route("/api/runs", get(list_runs))
        .route("/api/runs/compare", post(compare_runs))
        .route("/api/runs/{id}/start", post(start_run))
        .route("/api/runs/{id}/pause", post(pause_run))
        .route("/api/runs/{id}/step", post(step_run))
        .route("/api/runs/{id}/reset", post(reset_run))
        .route("/api/runs/{id}/policy", post(update_policy))
        .route("/api/runs/{id}/policy/next", post(apply_next_policy))
        .route(
            "/api/runs/{id}/risk/countermeasure",
            post(apply_countermeasure),
        )
        .route("/api/runs/{id}/policy-mode", post(update_policy_mode))
        .route("/api/runs/{id}/shock", post(inject_shock))
        .route("/api/runs/{id}/snapshot", get(snapshot))
        .route("/api/runs/{id}/history", get(history))
        .route("/api/runs/{id}/metrics", get(metrics))
        .route("/api/runs/{id}/metrics.parquet", get(metrics_parquet))
        .route("/api/runs/{id}/replay/{tick}", get(replay))
        .route("/api/runs/{id}/events", get(events))
        .route("/api/runs/{id}/transactions", get(transactions))
        .route(
            "/api/runs/{id}/compliance/{alert_id}/resolve",
            post(resolve_compliance),
        )
        .route("/api/runs/{id}/stream", get(stream))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    let port = std::env::var(PORT_ENV).unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn run_path(storage_dir: &FsPath, id: Uuid) -> PathBuf {
    storage_dir.join(format!("run-{id}.json"))
}

fn snapshot_log_path(storage_dir: &FsPath, id: Uuid) -> PathBuf {
    storage_dir.join(format!("run-{id}.snapshots.jsonl"))
}

fn snapshot_index_path(storage_dir: &FsPath, id: Uuid) -> PathBuf {
    storage_dir.join(format!("run-{id}.snapshots.index"))
}

#[derive(Serialize, Deserialize)]
struct PersistedRun {
    sim: Simulation,
    status: String,
    created_at: DateTime<Utc>,
}

fn load_snapshot_history(storage_dir: &FsPath, id: Uuid) -> Vec<Snapshot> {
    let path = snapshot_log_path(storage_dir, id);
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut history = Vec::new();
    for line in contents.lines() {
        if let Ok(snapshot) = serde_json::from_str::<Snapshot>(line) {
            history.push(snapshot);
            if history.len() > HISTORY_LIMIT {
                history.remove(0);
            }
        }
    }
    history
}

fn load_runs(storage_dir: &FsPath) -> HashMap<Uuid, RunRecord> {
    if let Err(error) = fs::create_dir_all(storage_dir) {
        eprintln!("could not create local run storage at {storage_dir:?}: {error}");
        return HashMap::new();
    }
    let mut runs = HashMap::new();
    let Ok(entries) = fs::read_dir(storage_dir) else {
        return runs;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(id_text) = name
            .strip_prefix("run-")
            .and_then(|name| name.strip_suffix(".json"))
        else {
            continue;
        };
        let Ok(id) = Uuid::parse_str(id_text) else {
            eprintln!("ignoring local run with invalid id: {path:?}");
            continue;
        };
        let Ok(bytes) = fs::read(&path) else {
            eprintln!("could not read local run: {path:?}");
            continue;
        };
        match serde_json::from_slice::<PersistedRun>(&bytes) {
            Ok(persisted) => {
                let mut history = load_snapshot_history(storage_dir, id);
                if history.is_empty() {
                    history = serde_json::from_slice::<RunRecord>(&bytes)
                        .map(|legacy| legacy.history)
                        .unwrap_or_default();
                }
                if history.is_empty() {
                    history.push(persisted.sim.snapshot());
                }
                runs.insert(
                    id,
                    RunRecord {
                        sim: persisted.sim,
                        status: persisted.status,
                        created_at: persisted.created_at,
                        history,
                    },
                );
            }
            Err(error) => eprintln!("ignoring corrupt local run {path:?}: {error}"),
        }
    }
    runs
}

fn persist_run(storage_dir: &FsPath, id: Uuid, record: &RunRecord) {
    if let Err(error) = fs::create_dir_all(storage_dir) {
        eprintln!("could not create local run storage at {storage_dir:?}: {error}");
        return;
    }
    let path = run_path(storage_dir, id);
    let temporary = path.with_extension("json.tmp");
    let persisted = PersistedRun {
        sim: record.sim.clone(),
        status: record.status.clone(),
        created_at: record.created_at,
    };
    match serde_json::to_vec(&persisted) {
        Ok(bytes) => {
            if let Err(error) =
                fs::write(&temporary, bytes).and_then(|_| fs::rename(&temporary, &path))
            {
                eprintln!("could not persist local run {path:?}: {error}");
                let _ = fs::remove_file(temporary);
            }
        }
        Err(error) => eprintln!("could not serialize local run {id}: {error}"),
    }
}

fn append_snapshot(storage_dir: &FsPath, id: Uuid, snapshot: &Snapshot) {
    if let Err(error) = fs::create_dir_all(storage_dir) {
        eprintln!("could not create snapshot storage at {storage_dir:?}: {error}");
        return;
    }
    let path = snapshot_log_path(storage_dir, id);
    let Ok(bytes) = serde_json::to_vec(snapshot) else {
        eprintln!("could not serialize snapshot for run {id}");
        return;
    };
    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)
    {
        Ok(mut file) => {
            let result = (|| {
                let offset = file.seek(SeekFrom::End(0))?;
                file.write_all(&bytes)?;
                file.write_all(b"\n")?;
                let index_path = snapshot_index_path(storage_dir, id);
                let mut index = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(index_path)?;
                writeln!(index, "{} {}", snapshot.tick, offset)
            })();
            if let Err(error) = result {
                eprintln!("could not append snapshot {path:?}: {error}");
            }
        }
        Err(error) => eprintln!("could not open snapshot log {path:?}: {error}"),
    }
}

fn truncate_snapshot_log(storage_dir: &FsPath, id: Uuid) {
    let path = snapshot_log_path(storage_dir, id);
    if let Err(error) = fs::write(&path, []) {
        eprintln!("could not reset snapshot log {path:?}: {error}");
    }
    let index_path = snapshot_index_path(storage_dir, id);
    if let Err(error) = fs::write(&index_path, []) {
        eprintln!("could not reset snapshot index {index_path:?}: {error}");
    }
}

fn load_snapshot_tick(storage_dir: &FsPath, id: Uuid, tick: u64) -> Option<Snapshot> {
    let path = snapshot_log_path(storage_dir, id);
    if let Ok(index) = fs::read_to_string(snapshot_index_path(storage_dir, id)) {
        if let Some(offset) = index.lines().rev().find_map(|line| {
            let mut fields = line.split_whitespace();
            (fields.next()?.parse::<u64>().ok()? == tick)
                .then(|| fields.next()?.parse::<u64>().ok())
                .flatten()
        }) {
            let mut file = fs::File::open(path).ok()?;
            file.seek(SeekFrom::Start(offset)).ok()?;
            let mut line = String::new();
            BufReader::new(file).read_line(&mut line).ok()?;
            return serde_json::from_str(&line).ok();
        }
    }
    let contents = fs::read_to_string(path).ok()?;
    contents
        .lines()
        .filter_map(|line| serde_json::from_str::<Snapshot>(line).ok())
        .find(|snapshot| snapshot.tick == tick)
}

fn load_metric_history(storage_dir: &FsPath, id: Uuid) -> Vec<MetricPoint> {
    let path = snapshot_log_path(storage_dir, id);
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };
    contents
        .lines()
        .filter_map(|line| serde_json::from_str::<PersistedMetricSnapshot>(line).ok())
        .map(|snapshot| MetricPoint {
            tick: snapshot.tick,
            metrics: snapshot.metrics,
        })
        .collect()
}

async fn create_run(
    State(state): State<AppState>,
    Json(config): Json<RunConfig>,
) -> Json<RunCreated> {
    let sim = Simulation::new(config);
    let initial_snapshot = sim.snapshot();
    let id = Uuid::new_v4();
    let created_at = Utc::now();
    let response = RunCreated {
        run_id: id,
        status: "paused".into(),
        created_at,
        config_hash: sim.config_hash(),
    };
    let record = RunRecord {
        sim,
        status: "paused".into(),
        created_at,
        history: vec![initial_snapshot],
    };
    persist_run(&state.storage_dir, id, &record);
    append_snapshot(&state.storage_dir, id, &record.history[0]);
    state.runs.write().await.insert(id, record);
    Json(response)
}

async fn list_runs(State(state): State<AppState>) -> Json<Vec<RunSummary>> {
    let runs = state.runs.read().await;
    let mut summaries: Vec<_> = runs
        .iter()
        .map(|(id, record)| RunSummary {
            run_id: *id,
            status: record.status.clone(),
            tick: record.sim.tick,
            created_at: record.created_at,
            config_hash: record.sim.config_hash(),
            history_ticks: record
                .history
                .iter()
                .map(|snapshot| snapshot.tick)
                .collect(),
        })
        .collect();
    summaries.sort_by_key(|summary| summary.created_at);
    Json(summaries)
}

async fn compare_runs(
    State(state): State<AppState>,
    Json(request): Json<CompareRequest>,
) -> impl IntoResponse {
    let runs = state.runs.read().await;
    let mut rows = Vec::new();
    for run_id in request.run_ids {
        let Some(record) = runs.get(&run_id) else {
            continue;
        };
        let selected = request.tick.and_then(|tick| {
            record
                .history
                .iter()
                .find(|snapshot| snapshot.tick == tick)
                .cloned()
                .or_else(|| load_snapshot_tick(&state.storage_dir, run_id, tick))
        });
        let selected = selected.or_else(|| record.history.last().cloned());
        if let Some(snapshot) = selected {
            rows.push(ComparisonRow {
                run_id,
                tick: snapshot.tick,
                metrics: snapshot.metrics.clone(),
            });
        }
    }
    Json(ComparisonResponse {
        requested_tick: request.tick,
        runs: rows,
    })
}

fn remember(record: &mut RunRecord) {
    record.history.push(record.sim.snapshot());
    if record.history.len() > HISTORY_LIMIT {
        let excess = record.history.len() - HISTORY_LIMIT;
        record.history.drain(..excess);
    }
}
async fn start_run(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(r) => {
            r.status = "running".into();
            persist_run(&state.storage_dir, id, r);
            Json(StatusResponse {
                run_id: id,
                status: r.status.clone(),
                tick: r.sim.tick,
            })
            .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn pause_run(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(r) => {
            r.status = "paused".into();
            persist_run(&state.storage_dir, id, r);
            Json(StatusResponse {
                run_id: id,
                status: r.status.clone(),
                tick: r.sim.tick,
            })
            .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn step_run(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(r) => {
            let snapshot = r.sim.step();
            r.history.push(snapshot.clone());
            if r.history.len() > HISTORY_LIMIT {
                let excess = r.history.len() - HISTORY_LIMIT;
                r.history.drain(..excess);
            }
            persist_run(&state.storage_dir, id, r);
            append_snapshot(&state.storage_dir, id, &snapshot);
            Json(snapshot).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn reset_run(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(r) => {
            r.sim = Simulation::new(r.sim.config.clone());
            r.status = "paused".into();
            r.history.clear();
            r.history.push(r.sim.snapshot());
            truncate_snapshot_log(&state.storage_dir, id);
            persist_run(&state.storage_dir, id, r);
            append_snapshot(&state.storage_dir, id, &r.history[0]);
            Json(StatusResponse {
                run_id: id,
                status: r.status.clone(),
                tick: r.sim.tick,
            })
            .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn update_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(policy): Json<PolicyConfig>,
) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(r) => {
            r.sim.apply_policy(policy);
            remember(r);
            persist_run(&state.storage_dir, id, r);
            append_snapshot(
                &state.storage_dir,
                id,
                r.history.last().expect("history is non-empty"),
            );
            Json(r.sim.snapshot()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
#[derive(Deserialize)]
struct PolicyModeRequest {
    mode: PolicyMode,
}

#[derive(Deserialize)]
struct ShockRequest {
    kind: String,
    region: u16,
    intensity: f64,
}

async fn apply_next_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(r) => {
            if r.sim.apply_next_policy() {
                remember(r);
                persist_run(&state.storage_dir, id, r);
                append_snapshot(
                    &state.storage_dir,
                    id,
                    r.history.last().expect("history is non-empty"),
                );
                Json(r.sim.snapshot()).into_response()
            } else {
                StatusCode::CONFLICT.into_response()
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn update_policy_mode(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<PolicyModeRequest>,
) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(r) => {
            r.sim.config.policy_mode = request.mode;
            remember(r);
            persist_run(&state.storage_dir, id, r);
            append_snapshot(
                &state.storage_dir,
                id,
                r.history.last().expect("history is non-empty"),
            );
            Json(r.sim.snapshot()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn inject_shock(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ShockRequest>,
) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(record) => {
            if !record
                .sim
                .inject_external_shock(&request.kind, request.region, request.intensity)
            {
                return StatusCode::UNPROCESSABLE_ENTITY.into_response();
            }
            remember(record);
            persist_run(&state.storage_dir, id, record);
            append_snapshot(
                &state.storage_dir,
                id,
                record.history.last().expect("history is non-empty"),
            );
            Json(record.sim.snapshot()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn apply_countermeasure(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(r) => {
            if r.sim.apply_countermeasure() {
                remember(r);
                persist_run(&state.storage_dir, id, r);
                append_snapshot(
                    &state.storage_dir,
                    id,
                    r.history.last().expect("history is non-empty"),
                );
                Json(r.sim.snapshot()).into_response()
            } else {
                StatusCode::CONFLICT.into_response()
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn history(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let runs = state.runs.read().await;
    runs.get(&id)
        .map(|record| Json(record.history.clone()).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

async fn metrics(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let runs = state.runs.read().await;
    let Some(record) = runs.get(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let points = metric_points_for_run(&state.storage_dir, record, id);
    Json(points).into_response()
}

fn metric_points_for_run(storage_dir: &FsPath, record: &RunRecord, id: Uuid) -> Vec<MetricPoint> {
    let mut points = load_metric_history(storage_dir, id);
    if points.is_empty() {
        points = record
            .history
            .iter()
            .map(|snapshot| MetricPoint {
                tick: snapshot.tick,
                metrics: snapshot.metrics.clone(),
            })
            .collect();
    }
    points
}

fn write_metrics_parquet(points: &[MetricPoint]) -> Result<Vec<u8>, String> {
    let schema = std::sync::Arc::new(Schema::new(vec![
        Field::new("tick", DataType::UInt64, false),
        Field::new("gdp", DataType::Float64, false),
        Field::new("gini", DataType::Float64, false),
        Field::new("unemployment", DataType::Float64, false),
        Field::new("inflation", DataType::Float64, false),
        Field::new("tax_revenue", DataType::Float64, false),
        Field::new("trade_volume", DataType::Float64, false),
        Field::new("alerts", DataType::UInt64, false),
        Field::new("mean_wealth", DataType::Float64, false),
        Field::new("money_supply", DataType::Float64, false),
        Field::new("interest_rate", DataType::Float64, false),
        Field::new("top_10_wealth_share", DataType::Float64, false),
        Field::new("money_created_this_tick", DataType::Float64, false),
        Field::new("cumulative_money_created", DataType::Float64, false),
        Field::new("government_debt", DataType::Float64, false),
        Field::new("private_debt", DataType::Float64, false),
        Field::new("debt_created_this_tick", DataType::Float64, false),
        Field::new("wealth_shock", DataType::Float64, false),
        Field::new("adverse_events", DataType::UInt64, false),
        Field::new("open_risks", DataType::UInt64, false),
        Field::new("countermeasures", DataType::UInt64, false),
        Field::new("systemic_risk", DataType::Float64, false),
        Field::new("security_level", DataType::Float64, false),
        Field::new("aml_alerts", DataType::UInt64, false),
        Field::new("tax_alerts", DataType::UInt64, false),
    ]));
    let ticks = UInt64Array::from_iter_values(points.iter().map(|point| point.tick));
    let f64_column = |value: fn(&globaleconomy::Metrics) -> f64| -> ArrayRef {
        std::sync::Arc::new(Float64Array::from_iter_values(
            points.iter().map(|point| value(&point.metrics)),
        ))
    };
    let u64_column = |value: fn(&globaleconomy::Metrics) -> u64| -> ArrayRef {
        std::sync::Arc::new(UInt64Array::from_iter_values(
            points.iter().map(|point| value(&point.metrics)),
        ))
    };
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            std::sync::Arc::new(ticks),
            f64_column(|m| m.gdp),
            f64_column(|m| m.gini),
            f64_column(|m| m.unemployment),
            f64_column(|m| m.inflation),
            f64_column(|m| m.tax_revenue),
            f64_column(|m| m.trade_volume),
            u64_column(|m| m.alerts),
            f64_column(|m| m.mean_wealth),
            f64_column(|m| m.money_supply),
            f64_column(|m| m.interest_rate),
            f64_column(|m| m.top_10_wealth_share),
            f64_column(|m| m.money_created_this_tick),
            f64_column(|m| m.cumulative_money_created),
            f64_column(|m| m.government_debt),
            f64_column(|m| m.private_debt),
            f64_column(|m| m.debt_created_this_tick),
            f64_column(|m| m.wealth_shock),
            u64_column(|m| m.adverse_events),
            u64_column(|m| m.open_risks),
            u64_column(|m| m.countermeasures),
            f64_column(|m| m.systemic_risk),
            f64_column(|m| m.security_level),
            u64_column(|m| m.aml_alerts),
            u64_column(|m| m.tax_alerts),
        ],
    )
    .map_err(|error| format!("could not build metric batch: {error}"))?;
    let mut bytes = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut bytes, schema, None)
        .map_err(|error| format!("could not create parquet writer: {error}"))?;
    writer
        .write(&batch)
        .map_err(|error| format!("could not write parquet batch: {error}"))?;
    writer
        .close()
        .map_err(|error| format!("could not close parquet writer: {error}"))?;
    Ok(bytes)
}

async fn metrics_parquet(State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    let runs = state.runs.read().await;
    let Some(record) = runs.get(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let points = metric_points_for_run(&state.storage_dir, record, id);
    match write_metrics_parquet(&points) {
        Ok(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, "application/vnd.apache.parquet")
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"run-{id}-metrics.parquet\""),
            )
            .body(Body::from(bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(error) => {
            eprintln!("could not export parquet for run {id}: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn replay(
    State(state): State<AppState>,
    Path((id, tick)): Path<(Uuid, u64)>,
) -> impl IntoResponse {
    let runs = state.runs.read().await;
    let Some(record) = runs.get(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    record
        .history
        .iter()
        .find(|snapshot| snapshot.tick == tick)
        .cloned()
        .or_else(|| load_snapshot_tick(&state.storage_dir, id, tick))
        .map(|snapshot| Json(snapshot).into_response())
        .unwrap_or_else(|| StatusCode::GONE.into_response())
}
async fn snapshot(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let runs = state.runs.read().await;
    runs.get(&id)
        .map(|r| Json(r.sim.snapshot()).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}
fn filter_events(sim: &Simulation, query: &EventQuery) -> Vec<EconomicEvent> {
    sim.events
        .iter()
        .filter(|event| query.tick.is_none_or(|tick| event.tick == tick))
        .filter(|event| {
            query
                .event_type
                .as_ref()
                .is_none_or(|kind| &event.event_type == kind)
        })
        .filter(|event| query.agent.is_none_or(|agent| event.agent_id == agent))
        .filter(|event| query.region.is_none_or(|region| event.region == region))
        .filter(|event| query.min_amount.is_none_or(|amount| event.amount >= amount))
        .filter(|event| query.max_amount.is_none_or(|amount| event.amount <= amount))
        .filter(|event| {
            !query.risk.unwrap_or(false)
                || sim
                    .people
                    .iter()
                    .find(|person| person.id == event.agent_id)
                    .is_some_and(|person| person.risk_score >= 0.8)
        })
        .cloned()
        .collect()
}

fn filter_transactions(sim: &Simulation, query: &TransactionQuery) -> Vec<Transaction> {
    sim.transactions
        .iter()
        .filter(|transaction| query.tick.is_none_or(|tick| transaction.tick == tick))
        .filter(|transaction| {
            query
                .kind
                .as_ref()
                .is_none_or(|kind| &transaction.kind == kind)
        })
        .filter(|transaction| {
            query
                .agent
                .is_none_or(|agent| transaction.from == agent || transaction.to == agent)
        })
        .filter(|transaction| {
            query
                .region
                .is_none_or(|region| transaction.region == region)
        })
        .filter(|transaction| {
            query
                .min_amount
                .is_none_or(|amount| transaction.amount >= amount)
        })
        .filter(|transaction| {
            query
                .max_amount
                .is_none_or(|amount| transaction.amount <= amount)
        })
        .cloned()
        .collect()
}

async fn events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<EventQuery>,
) -> impl IntoResponse {
    let runs = state.runs.read().await;
    runs.get(&id)
        .map(|r| Json(filter_events(&r.sim, &query)).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

async fn transactions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<TransactionQuery>,
) -> impl IntoResponse {
    let runs = state.runs.read().await;
    runs.get(&id)
        .map(|r| Json(filter_transactions(&r.sim, &query)).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

async fn resolve_compliance(
    State(state): State<AppState>,
    Path((id, alert_id)): Path<(Uuid, u64)>,
) -> impl IntoResponse {
    let mut runs = state.runs.write().await;
    match runs.get_mut(&id) {
        Some(record) => {
            if !record.sim.resolve_compliance_alert(alert_id) {
                return StatusCode::CONFLICT.into_response();
            }
            remember(record);
            persist_run(&state.storage_dir, id, record);
            append_snapshot(
                &state.storage_dir,
                id,
                record.history.last().expect("history is non-empty"),
            );
            Json(record.sim.snapshot()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn stream(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| stream_socket(socket, state, id))
}
async fn stream_socket(mut socket: WebSocket, state: AppState, id: Uuid) {
    loop {
        let (status, snap): (String, Snapshot) = {
            let mut runs = state.runs.write().await;
            let Some(r) = runs.get_mut(&id) else {
                return;
            };
            if r.status == "running" {
                r.sim.step();
                remember(r);
                persist_run(&state.storage_dir, id, r);
                append_snapshot(
                    &state.storage_dir,
                    id,
                    r.history.last().expect("history is non-empty"),
                );
            }
            (r.status.clone(), r.sim.snapshot())
        };
        let payload = serde_json::json!({"type":"snapshot","status":&status,"data":&snap});
        if socket
            .send(Message::Text(payload.to_string().into()))
            .await
            .is_err()
        {
            return;
        }
        let metric_payload = serde_json::json!({
            "type": "metric_update",
            "data": {"tick": snap.tick, "metrics": &snap.metrics}
        });
        if socket
            .send(Message::Text(metric_payload.to_string().into()))
            .await
            .is_err()
        {
            return;
        }
        let transaction_payload = serde_json::json!({
            "type": "transaction",
            "data": snap.transactions.iter().take(100).collect::<Vec<_>>()
        });
        if socket
            .send(Message::Text(transaction_payload.to_string().into()))
            .await
            .is_err()
        {
            return;
        }
        let alert_payload = serde_json::json!({
            "type": "alert",
            "data": snap
                .compliance_alerts
                .iter()
                .filter(|alert| alert.tick == snap.tick.saturating_sub(1))
                .collect::<Vec<_>>()
        });
        if socket
            .send(Message::Text(alert_payload.to_string().into()))
            .await
            .is_err()
        {
            return;
        }
        let status_payload = serde_json::json!({
            "type": "run_status",
            "data": {"run_id": id, "status": &status, "tick": snap.tick}
        });
        if socket
            .send(Message::Text(status_payload.to_string().into()))
            .await
            .is_err()
        {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_run_round_trips_simulation_and_history() {
        let sim = Simulation::new(RunConfig::default());
        let record = RunRecord {
            history: vec![sim.snapshot()],
            sim,
            status: "paused".into(),
            created_at: Utc::now(),
        };

        let bytes = serde_json::to_vec(&record).expect("run should serialize");
        let restored: RunRecord = serde_json::from_slice(&bytes).expect("run should deserialize");

        assert_eq!(restored.sim.tick, record.sim.tick);
        assert_eq!(restored.sim.people.len(), record.sim.people.len());
        assert_eq!(restored.history.len(), 1);
        assert_eq!(restored.history[0].tick, 0);
        assert_eq!(restored.status, "paused");
    }

    #[test]
    fn snapshot_log_supports_exact_tick_lookup() {
        let directory = std::env::temp_dir().join(format!("globaleconomy-{}", Uuid::new_v4()));
        let id = Uuid::new_v4();
        let mut sim = Simulation::new(RunConfig::default());
        let first = sim.snapshot();
        let second = sim.step();

        append_snapshot(&directory, id, &first);
        append_snapshot(&directory, id, &second);

        let loaded = load_snapshot_tick(&directory, id, 1).expect("tick should be indexed");
        assert_eq!(loaded.tick, second.tick);
        assert_eq!(load_snapshot_history(&directory, id).len(), 2);
        let index = fs::read_to_string(snapshot_index_path(&directory, id))
            .expect("snapshot offset index should be persisted");
        assert_eq!(index.lines().count(), 2);
        fs::remove_dir_all(directory).expect("temporary persistence directory should be removable");
    }

    #[test]
    fn snapshot_log_produces_compact_metric_history() {
        let directory = std::env::temp_dir().join(format!("globaleconomy-{}", Uuid::new_v4()));
        let id = Uuid::new_v4();
        let mut sim = Simulation::new(RunConfig::default());
        let first = sim.snapshot();
        let second = sim.step();

        append_snapshot(&directory, id, &first);
        append_snapshot(&directory, id, &second);

        let points = load_metric_history(&directory, id);
        assert_eq!(
            points.iter().map(|point| point.tick).collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(points[1].metrics.gdp, second.metrics.gdp);
        fs::remove_dir_all(directory).expect("temporary persistence directory should be removable");
    }

    #[test]
    fn metric_history_exports_valid_parquet_magic() {
        let mut sim = Simulation::new(RunConfig::default());
        let points = vec![
            MetricPoint {
                tick: 0,
                metrics: sim.snapshot().metrics,
            },
            MetricPoint {
                tick: 1,
                metrics: sim.step().metrics,
            },
        ];
        let bytes = write_metrics_parquet(&points).expect("metrics should export");
        assert_eq!(&bytes[..4], b"PAR1");
        assert_eq!(&bytes[bytes.len() - 4..], b"PAR1");
    }

    #[test]
    fn event_and_transaction_filters_select_auditable_ranges() {
        let mut sim = Simulation::new(RunConfig {
            agent_count: 8,
            regions: 2,
            ..RunConfig::default()
        });
        sim.people[0].risk_score = 0.95;
        sim.events = vec![
            EconomicEvent {
                tick: 1,
                event_type: "purchase".into(),
                agent_id: 0,
                region: 1,
                amount: 250.0,
                reason: "test".into(),
            },
            EconomicEvent {
                tick: 2,
                event_type: "purchase".into(),
                agent_id: 1,
                region: 0,
                amount: 800.0,
                reason: "test".into(),
            },
        ];
        sim.transactions = vec![
            Transaction {
                id: 1,
                tick: 1,
                from: 0,
                to: 3_000_000,
                kind: "purchase".into(),
                amount: 250.0,
                region: 1,
                reason: "test".into(),
            },
            Transaction {
                id: 2,
                tick: 2,
                from: 4_000_000,
                to: 1,
                kind: "loan".into(),
                amount: 900.0,
                region: 0,
                reason: "test".into(),
            },
        ];

        let events = filter_events(
            &sim,
            &EventQuery {
                event_type: Some("purchase".into()),
                tick: Some(1),
                risk: Some(true),
                ..Default::default()
            },
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].agent_id, 0);

        let transactions = filter_transactions(
            &sim,
            &TransactionQuery {
                kind: Some("loan".into()),
                min_amount: Some(800.0),
                agent: Some(1),
                ..Default::default()
            },
        );
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id, 2);
    }
}
