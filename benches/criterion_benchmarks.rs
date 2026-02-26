// Criterion benchmarks for fast micro-benchmarks of individual functions
// useful for testing the performance of specific code and logic optimizations

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use n_body_sim::args::{Args, GravityMethod, IntegratorMethod};
use n_body_sim::simulation::{Body, Parameters, Simulator};

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

/// Common benchmark function for combinations of N, integrator method, and gravity method.
///
/// sample_size if set must be 10+, otherwise criterion will panic.
/// The total number of times sim.run() is called is sample_size * number of iterations per sample.
/// number of iterations per sample is determined by criterion based on the predicted time of each iteration,
/// and the measurement_time, where it will calculate enough iterations to take approximately measurement_time
/// for each benchmark. However it will run at least 1 iteration per sample, even if the sim takes a long time
/// which causes the benchmark to exceed the measurement_time. To run benchmarks as fast as possible, set a
/// short measurement_time, so that it's preductions lead it to run only 1 iteration per sample (for 10 samples minimum)
/// which means the least possible amount of iterations per benchmark is 10
fn bench(
    c: &mut Criterion,
    gravity_methods: &[GravityMethod],
    integrator_methods: &[IntegratorMethod],
    n_values: &[usize],
    sample_size: Option<usize>,
    measurement_time: Option<std::time::Duration>,
    batch_size: Option<BatchSize>,
) {
    let args = Args::default();
    let mut group = c.benchmark_group("N-Body-Simulation");

    if let Some(size) = sample_size {
        assert!(size >= 10);
        group.sample_size(size);
    }

    if let Some(time) = measurement_time {
        group.measurement_time(time);
    }

    for &gravity_method in gravity_methods {
        for &integrator_method in integrator_methods {
            for &n in n_values {
                group.bench_with_input(
                    BenchmarkId::new(
                        format!("{:?}/{:?}", gravity_method, integrator_method),
                        format!("{}", n),
                    ),
                    &(gravity_method, integrator_method, n),
                    |b, &(gravity_method, integrator_method, n)| {
                        b.iter_batched_ref(
                            || create_simulation(n, gravity_method, integrator_method, &args),
                            |sim| sim.run(),
                            batch_size.unwrap_or(BatchSize::SmallInput),
                        );
                    },
                );
            }
        }
    }
}

// Benchmark for Newton gravity method
#[allow(dead_code)]
fn bench_newton(c: &mut Criterion) {
    const N_VALUES: [usize; 10] = [2, 3, 5, 10, 25, 50, 75, 100, 150, 200];

    bench(
        c,
        &[GravityMethod::Newton],
        &[IntegratorMethod::Euler],
        &N_VALUES,
        Some(10),                                // sample size
        Some(std::time::Duration::from_secs(5)), // measurement time
        Some(BatchSize::PerIteration),           // batch size
    );
}

// Benchmark for Newton gravity method
#[allow(dead_code)]
fn bench_newton_parallel(c: &mut Criterion) {
    const N_VALUES: [usize; 10] = [2, 3, 5, 10, 25, 50, 75, 100, 150, 200];

    bench(
        c,
        &[GravityMethod::NewtonParallel],
        &[IntegratorMethod::Euler],
        &N_VALUES,
        Some(10),                                // sample size
        Some(std::time::Duration::from_secs(5)), // measurement time
        Some(BatchSize::PerIteration),           // batch size
    );
}

// Benchmark for Barnes-Hut gravity method
#[allow(dead_code)]
fn bench_barnes_hut(c: &mut Criterion) {
    const N_VALUES: [usize; 10] = [2, 3, 5, 10, 25, 50, 75, 100, 150, 200];

    bench(
        c,
        &[GravityMethod::BarnesHut],
        &[IntegratorMethod::Euler],
        &N_VALUES,
        Some(10),                                // sample size
        Some(std::time::Duration::from_secs(5)), // measurement time
        Some(BatchSize::PerIteration),           // batch size
    );
}

// Benchmark for Newton vs NewtonParallel gravity methods with Euler integrator
#[allow(dead_code)]
fn bench_newton_vs_newton_parallel(c: &mut Criterion) {
    const N_VALUES: [usize; 10] = [2, 3, 5, 10, 25, 50, 75, 100, 150, 200];

    bench(
        c,
        &[GravityMethod::Newton, GravityMethod::NewtonParallel],
        &[IntegratorMethod::Euler],
        &N_VALUES,
        Some(10),                                // sample size
        Some(std::time::Duration::from_secs(5)), // measurement time
        Some(BatchSize::PerIteration),           // batch size
    );
}

// Benchmark for NewtonParallel vs BarnesHut gravity methods with Euler integrator
#[allow(dead_code)]
fn bench_newton_parallel_vs_barnes_hut(c: &mut Criterion) {
    const N_VALUES: [usize; 10] = [2, 3, 5, 10, 25, 50, 75, 100, 150, 200];

    bench(
        c,
        &[GravityMethod::NewtonParallel, GravityMethod::BarnesHut],
        &[IntegratorMethod::Euler],
        &N_VALUES,
        Some(10),                                // sample size
        Some(std::time::Duration::from_secs(5)), // measurement time
        Some(BatchSize::PerIteration),           // batch size
    );
}

// Benchmark for all gravity methods with Euler integrator
#[allow(dead_code)]
fn bench_all_gravity_methods(c: &mut Criterion) {
    const N_VALUES: [usize; 10] = [2, 3, 5, 10, 25, 50, 75, 100, 150, 200];

    bench(
        c,
        &[
            GravityMethod::Newton,
            GravityMethod::NewtonParallel,
            GravityMethod::BarnesHut,
        ],
        &[IntegratorMethod::Euler],
        &N_VALUES,
        Some(10),                                // sample size
        Some(std::time::Duration::from_secs(5)), // measurement time
        Some(BatchSize::PerIteration),           // batch size
    );
}

criterion_group!(
    benches, // macro-generated "benches" method
    bench_newton_vs_newton_parallel,
);
criterion_main!(benches);
