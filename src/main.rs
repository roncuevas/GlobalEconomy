use axum::{
    extract::ws::{Message, WebSocket},
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use globaleconomy::{PolicyConfig, PolicyMode, RunConfig, Simulation, Snapshot};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Write,
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

const HISTORY_LIMIT: usize = 240;
const DATA_DIR_ENV: &str = "GLOBALECONOMY_DATA_DIR";

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
        .route("/api/runs/{id}/snapshot", get(snapshot))
        .route("/api/runs/{id}/history", get(history))
        .route("/api/runs/{id}/metrics", get(metrics))
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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn run_path(storage_dir: &FsPath, id: Uuid) -> PathBuf {
    storage_dir.join(format!("run-{id}.json"))
}

fn snapshot_log_path(storage_dir: &FsPath, id: Uuid) -> PathBuf {
    storage_dir.join(format!("run-{id}.snapshots.jsonl"))
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
    match fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => {
            if let Err(error) = file.write_all(&bytes).and_then(|_| file.write_all(b"\n")) {
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
}

fn load_snapshot_tick(storage_dir: &FsPath, id: Uuid, tick: u64) -> Option<Snapshot> {
    let path = snapshot_log_path(storage_dir, id);
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
    let mut points = load_metric_history(&state.storage_dir, id);
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
    Json(points).into_response()
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
async fn events(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let runs = state.runs.read().await;
    runs.get(&id)
        .map(|r| Json(r.sim.events.clone()).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

async fn transactions(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let runs = state.runs.read().await;
    runs.get(&id)
        .map(|r| Json(r.sim.transactions.clone()).into_response())
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
        let payload = serde_json::json!({"type":"snapshot","status":status,"data":snap});
        if socket
            .send(Message::Text(payload.to_string().into()))
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
}
