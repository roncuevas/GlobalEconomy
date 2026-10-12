use globaleconomy::{RunConfig, Simulation};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    println!("GlobalEconomy engine scale benchmark (one deterministic tick)");
    println!("agents,init_ms,step_ms,visual_agents,transactions");

    for agent_count in [1_000_usize, 10_000_usize] {
        let mut config = RunConfig {
            agent_count,
            ticks: 1,
            catastrophes: false,
            ..RunConfig::default()
        };
        config.seed = 42;

        let init_start = Instant::now();
        let mut simulation = Simulation::new(config);
        let init_ms = init_start.elapsed().as_secs_f64() * 1_000.0;

        let step_start = Instant::now();
        let snapshot = black_box(simulation.step());
        let step_ms = step_start.elapsed().as_secs_f64() * 1_000.0;

        println!(
            "{agent_count},{init_ms:.3},{step_ms:.3},{},{}",
            snapshot.agents.len(),
            simulation.transactions.len()
        );
    }
}
