//! Rutas del modo «Generaciones». Las corridas viven solo en memoria: el motor es determinista,
//! así que la UI puede recrear una corrida perdida con la misma configuración y `advance_to`.

use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use globaleconomy::generations::{GenConfig, GenEvent, GenSim, GenSnapshot};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

const MAX_RUNS: usize = 32;
const MAX_STEP: u64 = 50;

pub struct GenRecord {
    sim: GenSim,
    created_at: DateTime<Utc>,
}

pub type GenStore = Arc<RwLock<HashMap<Uuid, GenRecord>>>;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct CreateRequest {
    config: GenConfig,
    advance_to: Option<u64>,
}

#[derive(Serialize)]
pub struct Created {
    id: Uuid,
    config_hash: String,
    snapshot: GenSnapshot,
}

#[derive(Deserialize)]
pub struct StepRequest {
    #[serde(default = "one")]
    count: u64,
}

fn one() -> u64 {
    1
}

#[derive(Serialize)]
pub struct Stepped {
    snapshot: GenSnapshot,
    events: Vec<GenEvent>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/generations", post(create))
        .route("/api/generations/{id}/step", post(step))
        .route("/api/generations/{id}/reset", post(reset))
        .route("/api/generations/{id}/snapshot", get(snapshot))
        .route("/api/generations/{id}/person/{pid}", get(person))
}

async fn create(
    State(state): State<AppState>,
    Json(request): Json<CreateRequest>,
) -> Json<Created> {
    let mut sim = GenSim::new(request.config);
    let target = request.advance_to.unwrap_or(0).min(sim.config.max_ticks);
    while sim.tick < target && !sim.finished() {
        sim.step();
    }
    let id = Uuid::new_v4();
    let created = Created {
        id,
        config_hash: sim.config_hash(),
        snapshot: sim.snapshot(),
    };
    let mut runs = state.generations.write().await;
    if runs.len() >= MAX_RUNS {
        if let Some(oldest) = runs
            .iter()
            .min_by_key(|(_, r)| r.created_at)
            .map(|(id, _)| *id)
        {
            runs.remove(&oldest);
        }
    }
    runs.insert(
        id,
        GenRecord {
            sim,
            created_at: Utc::now(),
        },
    );
    Json(created)
}

async fn step(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<StepRequest>,
) -> Response {
    let mut runs = state.generations.write().await;
    let Some(record) = runs.get_mut(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut events = Vec::new();
    for _ in 0..request.count.clamp(1, MAX_STEP) {
        if record.sim.finished() {
            break;
        }
        events.extend(record.sim.step());
    }
    Json(Stepped {
        snapshot: record.sim.snapshot(),
        events,
    })
    .into_response()
}

async fn reset(State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    let mut runs = state.generations.write().await;
    let Some(record) = runs.get_mut(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    record.sim = GenSim::new(record.sim.config.clone());
    Json(record.sim.snapshot()).into_response()
}

async fn snapshot(State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    let runs = state.generations.read().await;
    runs.get(&id)
        .map(|r| Json(r.sim.snapshot()).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

async fn person(State(state): State<AppState>, Path((id, pid)): Path<(Uuid, u32)>) -> Response {
    let runs = state.generations.read().await;
    runs.get(&id)
        .and_then(|r| r.sim.family(pid))
        .map(|family| Json(family).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn state() -> AppState {
        AppState {
            runs: Arc::new(RwLock::new(HashMap::new())),
            storage_dir: Arc::new(PathBuf::from("unused")),
            generations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[tokio::test]
    async fn create_step_and_inspect_a_society() {
        let state = state();
        let Json(created) = create(State(state.clone()), Json(CreateRequest::default())).await;
        assert_eq!(created.snapshot.tick, 0);
        assert_eq!(created.snapshot.metrics.population, 12);
        let response = step(
            State(state.clone()),
            Path(created.id),
            Json(StepRequest { count: 10 }),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let runs = state.generations.read().await;
        let sim = &runs[&created.id].sim;
        assert_eq!(sim.tick, 10);
        assert!(sim.family(0).is_some());
    }

    #[tokio::test]
    async fn advance_to_recreates_the_same_society() {
        let state = state();
        let config = GenConfig {
            seed: 9,
            ..GenConfig::default()
        };
        let Json(stepped) = create(
            State(state.clone()),
            Json(CreateRequest {
                config: config.clone(),
                advance_to: None,
            }),
        )
        .await;
        for _ in 0..25 {
            step(
                State(state.clone()),
                Path(stepped.id),
                Json(StepRequest { count: 1 }),
            )
            .await;
        }
        let Json(recreated) = create(
            State(state.clone()),
            Json(CreateRequest {
                config,
                advance_to: Some(25),
            }),
        )
        .await;
        let runs = state.generations.read().await;
        let a = serde_json::to_string(&runs[&stepped.id].sim.snapshot()).unwrap();
        let b = serde_json::to_string(&recreated.snapshot).unwrap();
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn unknown_society_is_not_found() {
        let response = snapshot(State(state()), Path(Uuid::new_v4())).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
