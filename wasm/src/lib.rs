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

#[cfg(test)]
mod tests {
    use super::WasmSimulation;
    use globaleconomy::RunConfig;

    #[test]
    fn json_adapter_creates_and_advances_small_run() {
        let mut config = RunConfig::default();
        config.agent_count = 16;
        config.ticks = 2;
        let config_json = serde_json::to_string(&config).expect("default config is serializable");
        let mut simulation = WasmSimulation::new(&config_json).expect("config should be accepted");

        assert_eq!(simulation.tick(), 0);
        let snapshot = simulation.step_json().expect("step should serialize");
        assert!(snapshot.contains("\"tick\":1"));
        assert_eq!(simulation.tick(), 1);
        assert!(simulation
            .snapshot_json()
            .expect("snapshot should serialize")
            .contains("\"agents\""));
    }
}
