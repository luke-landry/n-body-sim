use glam::DVec3;
use serde::Serialize;
use std::error::Error;
use std::time::Instant;

use crate::cli::{GravityMethod, IntegratorMethod};
use crate::simulation::{Body, Parameters, Simulator};

const N_VALUES: &[usize] = &[2, 3, 5, 10, 15, 25, 50, 75, 100, 200, 300];
const GRAVITY_METHODS: &[GravityMethod] = &[GravityMethod::Newton, GravityMethod::NewtonParallel];
const INTEGRATOR_METHODS: &[IntegratorMethod] =
    &[IntegratorMethod::Euler, IntegratorMethod::VelocityVerlet];
const NUMBER_OF_RUNS: usize = 5;

const NUM_STEPS: usize = 10000;
const TIME_STEP: f64 = 0.01;
const G_CONSTANT: f64 = 1.0;
const SOFTENING: f64 = 0.005;
const THETA: f64 = 0.5;
const PROGRESS: bool = false;

#[derive(Debug, Serialize)]
struct BenchmarkResult {
    n: usize,
    gravity: String,
    integrator: String,
    time_ms: f64,
}

fn save_results_to_csv(results: &[BenchmarkResult], path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    for result in results {
        wtr.serialize(result)?;
    }
    wtr.flush()?;
    Ok(())
}

fn generate_bodies(n: usize) -> Vec<Body> {
    (0..n)
        .map(|i| Body {
            id: i,
            mass: 1.0,
            position: DVec3::new(i as f64, 0.0, 0.0),
            velocity: DVec3::new(0.0, 1.0, 0.0),
        })
        .collect()
}

fn create_simulation(
    n: usize,
    gravity_method: GravityMethod,
    integrator_method: IntegratorMethod,
) -> Simulator {
    let bodies = generate_bodies(n);
    let parameters = Parameters {
        time_step: TIME_STEP,
        num_steps: NUM_STEPS,
        g_constant: G_CONSTANT,
        softening_factor: SOFTENING,
        theta: THETA,
        progress: PROGRESS,
    };
    let gravity = gravity_method.create(&parameters);
    let integrator = integrator_method.create(gravity, parameters.time_step, bodies.len());
    Simulator::new(bodies, parameters, integrator)
}

pub fn run_benchmark() -> Result<(), Box<dyn Error>> {
    println!(
        "{:>8} | {:>15} | {:>18} | {:>12}",
        "N", "Gravity", "Integrator", "Time (ms)"
    );
    println!("{}", "-".repeat(60));

    let mut warmup_sim = create_simulation(
        2,
        GravityMethod::NewtonParallel,
        IntegratorMethod::VelocityVerlet,
    );
    warmup_sim.run();

    let mut results = Vec::new();
    for &n in N_VALUES {
        for &gravity_method in GRAVITY_METHODS {
            for &integrator_method in INTEGRATOR_METHODS {
                for _ in 0..NUMBER_OF_RUNS {
                    let mut simulation = create_simulation(n, gravity_method, integrator_method);

                    let start_time = Instant::now();
                    simulation.run();
                    let duration = start_time.elapsed();

                    let time_ms = duration.as_secs_f64() * 1000.0;
                    let benchmark_result = BenchmarkResult {
                        n,
                        gravity: format!("{:?}", gravity_method),
                        integrator: format!("{:?}", integrator_method),
                        time_ms,
                    };

                    println!(
                        "{:>8} | {:>15} | {:>18} | {:>12.2}",
                        benchmark_result.n,
                        benchmark_result.gravity,
                        benchmark_result.integrator,
                        benchmark_result.time_ms
                    );
                    results.push(benchmark_result);
                }
            }
        }
    }

    save_results_to_csv(&results, "benchmark_results.csv")?;

    Ok(())
}
