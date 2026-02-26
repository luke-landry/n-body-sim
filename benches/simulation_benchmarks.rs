// Simulation benchmarks for long-running simulations with various configurations
// useful for testing the performance of different gravity and integrator methods
// Results are saved to the target/simulation_benchmarks directory as CSV files for easy analysis

use n_body_sim::args::{
    Args, GravityMethod,
    GravityMethod::{BarnesHut, Newton, NewtonParallel},
    IntegratorMethod,
    IntegratorMethod::{Euler, RungeKutta, VelocityVerlet},
};
use n_body_sim::simulation::{Body, Parameters, Simulator};
use serde::Serialize;
use std::error::Error;
use std::path::Path;
use std::time::Instant;

fn main() {
    if let Err(e) = run_benchmarks() {
        eprintln!("Error running benchmarks: {}", e);
        std::process::exit(1);
    }
}

fn run_benchmarks() -> Result<(), Box<dyn Error>> {
    // bench_newton_gravity()?;
    // bench_newton_parallel_gravity()?;
    // bench_barnes_hut_gravity()?;
    bench_all_gravity_methods()?;
    Ok(())
}

/*
    Benchmark functions
*/

/// Benchmark for Newton gravity method
#[allow(dead_code)]
fn bench_newton_gravity() -> Result<(), Box<dyn Error>> {
    bench(
        "Newton Gravity",                          // name
        &Args::default(),                          // args
        &[Newton],                                 // gravity methods
        &[Euler],                                  // integrator methods
        &[2, 5, 10],                               // n values
        3,                                         // sample size
        Path::new("benchmark_newton_results.csv"), // output path
    )
}

/// Benchmark for NewtonParallel gravity method
#[allow(dead_code)]
fn bench_newton_parallel_gravity() -> Result<(), Box<dyn Error>> {
    bench(
        "NewtonParallel Gravity",                           // name
        &Args::default(),                                   // args
        &[NewtonParallel],                                  // gravity methods
        &[Euler],                                           // integrator methods
        &[2, 5, 10],                                        // n values
        3,                                                  // sample size
        Path::new("benchmark_newton_parallel_results.csv"), // output path
    )
}

/// Benchmark for Barnes-Hut gravity method
#[allow(dead_code)]
fn bench_barnes_hut_gravity() -> Result<(), Box<dyn Error>> {
    bench(
        "Barnes-Hut Gravity",                          // name
        &Args::default(),                              // args
        &[BarnesHut],                                  // gravity methods
        &[Euler],                                      // integrator methods
        &[2, 5, 10],                                   // n values
        3,                                             // sample size
        Path::new("benchmark_barnes_hut_results.csv"), // output path
    )
}

/// Benchmark for all gravity methods
#[allow(dead_code)]
fn bench_all_gravity_methods() -> Result<(), Box<dyn Error>> {
    bench(
        "All Gravity Methods",                          // name
        &Args::default(),                               // args
        &[Newton, NewtonParallel, BarnesHut],           // gravity methods
        &[Euler],                                       // integrator methods
        &[2, 5, 10],                                    // n values
        3,                                              // sample size
        Path::new("benchmark_all_gravity_results.csv"), // output path
    )
}

/*
    Helper functions and structs for benchmarking
*/

fn generate_bodies(n: usize) -> Vec<Body> {
    (0..n)
        .map(|i| Body {
            mass: 1.0,
            pos_x: i as f64,
            pos_y: 0.0,
            pos_z: 0.0,
            vel_x: 0.0,
            vel_y: 1.0,
            vel_z: 0.0,
        })
        .collect()
}

fn create_simulator(
    n: usize,
    gravity_method: GravityMethod,
    integrator_method: IntegratorMethod,
    args: &Args,
) -> Simulator {
    let bodies = generate_bodies(n);
    let parameters = Parameters {
        time_step: args.time_step,
        num_steps: args.num_steps,
        g_constant: args.g_constant,
        softening_factor: args.softening_factor,
        theta: args.theta,
        progress: args.progress,
    };
    let gravity = gravity_method.create(&parameters, bodies.len());
    let integrator = integrator_method.create(gravity, parameters.time_step, bodies.len());
    Simulator::new(bodies, parameters, integrator, None)
}

#[derive(Debug, Serialize)]
struct BenchmarkResult {
    n: usize,
    gravity: String,
    integrator: String,
    time_ms: f64,
}

impl BenchmarkResult {
    fn new(n: usize, gravity: &str, integrator: &str, time_ms: f64) -> Self {
        Self {
            n,
            gravity: gravity.to_string(),
            integrator: integrator.to_string(),
            time_ms: time_ms,
        }
    }
}

fn save_results_to_csv(results: &[BenchmarkResult], path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut wtr = csv::Writer::from_path(path)?;
    for result in results {
        wtr.serialize(result)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn bench(
    name: &str,
    args: &Args,
    gravity_methods: &[GravityMethod],
    integrator_methods: &[IntegratorMethod],
    n_values: &[usize],
    sample_size: usize,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    println!("Running simulation benchmark: {}", name);
    println!(
        "{:>8} | {:>15} | {:>18} | {:>12}",
        "N", "Gravity", "Integrator", "Time (ms)"
    );
    println!("{}", "-".repeat(62));

    let mut benchmark_results = Vec::with_capacity(
        gravity_methods.len() * integrator_methods.len() * n_values.len() * sample_size,
    );

    for &gravity_method in gravity_methods {
        for &integrator_method in integrator_methods {
            for &n in n_values {
                for _ in 0..sample_size {
                    let mut simulation =
                        create_simulator(n, gravity_method, integrator_method, args);

                    let start_time = Instant::now();
                    simulation.run();
                    let duration = start_time.elapsed();

                    let benchmark_result = BenchmarkResult::new(
                        n,
                        &format!("{:?}", gravity_method),
                        &format!("{:?}", integrator_method),
                        duration.as_secs_f64() * 1000.0,
                    );

                    println!(
                        "{:>8} | {:>15} | {:>18} | {:>12.2}",
                        benchmark_result.n,
                        benchmark_result.gravity,
                        benchmark_result.integrator,
                        benchmark_result.time_ms
                    );

                    benchmark_results.push(benchmark_result);
                }
            }
        }
    }

    save_results_to_csv(
        &benchmark_results,
        &Path::new("target/simulation_benchmarks").join(output_path),
    )?;

    Ok(())
}
