use crate::cli::{Args, GravityMethod, IntegratorMethod};
use crate::simulation::{Body, Parameters, Simulator};
use serde::Serialize;
use std::error::Error;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Serialize)]
struct BenchmarkResult {
    n: usize,
    gravity: String,
    integrator: String,
    time_ms: f64,
}

fn save_results_to_csv(results: &[BenchmarkResult], path: &Path) -> Result<(), Box<dyn Error>> {
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

fn create_simulation(
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

fn run_benchmark(
    n: usize,
    gravity_method: GravityMethod,
    integrator_method: IntegratorMethod,
    args: &Args,
) -> BenchmarkResult {
    let mut simulation = create_simulation(n, gravity_method, integrator_method, args);
    let start_time = Instant::now();
    simulation.run();
    let duration = start_time.elapsed();
    BenchmarkResult {
        n,
        gravity: format!("{:?}", gravity_method),
        integrator: format!("{:?}", integrator_method),
        time_ms: duration.as_secs_f64() * 1000.0,
    }
}

pub fn run_benchmarks(args: Args) -> Result<(), Box<dyn Error>> {
    println!("Running n-body-sim benchmarks...");
    println!(
        "{:>8} | {:>15} | {:>18} | {:>12}",
        "N", "Gravity", "Integrator", "Time (ms)"
    );
    println!("{}", "-".repeat(62));

    let mut warmup_sim = create_simulation(
        2,
        GravityMethod::NewtonParallel,
        IntegratorMethod::VelocityVerlet,
        &args,
    );
    warmup_sim.run();

    let mut benchmark_results = Vec::new();
    for &n in args.benchmark_n_values.iter() {
        for &gravity_method in args.benchmark_gravity_methods.iter() {
            for &integrator_method in args.benchmark_integrator_methods.iter() {
                for _ in 0..args.benchmark_num_runs {
                    let benchmark_result =
                        run_benchmark(n, gravity_method, integrator_method, &args);
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

    save_results_to_csv(&benchmark_results, &args.benchmark_output_path)?;

    Ok(())
}
