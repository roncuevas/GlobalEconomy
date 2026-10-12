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
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    runs: Arc<RwLock<HashMap<Uuid, RunRecord>>>,
}
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

const HISTORY_LIMIT: usize = 240;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = AppState {
        runs: Arc::new(RwLock::new(HashMap::new())),
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
        .route("/api/runs/{id}/replay/{tick}", get(replay))
        .route("/api/runs/{id}/events", get(events))
        .route("/api/runs/{id}/transactions", get(transactions))
        .route("/api/runs/{id}/stream", get(stream))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
    state.runs.write().await.insert(
        id,
        RunRecord {
            sim,
            status: "paused".into(),
            created_at,
            history: vec![initial_snapshot],
        },
    );
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
        let selected = request
            .tick
            .and_then(|tick| record.history.iter().find(|snapshot| snapshot.tick == tick))
            .or_else(|| record.history.last());
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
        .map(|snapshot| Json(snapshot.clone()).into_response())
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
