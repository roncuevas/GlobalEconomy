use globaleconomy::{RunConfig, Simulation};
use wasm_bindgen::prelude::*;

/// Adaptador pequeño para ejecutar el mismo motor determinista dentro del navegador.
/// La interfaz usa JSON para mantener el contrato independiente del bundler web.
#[wasm_bindgen]
pub struct WasmSimulation {
    simulation: Simulation,
}

#[wasm_bindgen]
impl WasmSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<WasmSimulation, JsValue> {
        let config: RunConfig = serde_json::from_str(config_json)
            .map_err(|error| JsValue::from_str(&format!("invalid RunConfig: {error}")))?;
        Ok(Self {
            simulation: Simulation::new(config),
        })
    }

    pub fn step_json(&mut self) -> Result<String, JsValue> {
        serde_json::to_string(&self.simulation.step())
            .map_err(|error| JsValue::from_str(&format!("could not serialize snapshot: {error}")))
    }

    pub fn snapshot_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.simulation.snapshot())
            .map_err(|error| JsValue::from_str(&format!("could not serialize snapshot: {error}")))
    }

    pub fn tick(&self) -> u64 {
        self.simulation.tick
    }
}
