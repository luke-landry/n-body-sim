// Criterion benchmarks for fast micro-benchmarks of individual functions
// useful for testing the performance of specific code and logic optimizations

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use n_body_sim::args::{Args, GravityMethod};
use n_body_sim::simulation::Parameters;

/// Generates a non-trivial deterministic distribution of bodies to
/// for more realistic and consistent performance during benchmarks
fn generate_distributed_bodies_positions(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut masses = Vec::with_capacity(n);
    let mut rx = Vec::with_capacity(n);
    let mut ry = Vec::with_capacity(n);
    let mut rz = Vec::with_capacity(n);

    // 20 x 20 x 20 bounding box
    let radius = 10.0;
    let height = 20.0;

    let mass_base = 1.0;

    for i in 0..n {
        // Spiral distribution in x/y, layered in z
        let angle = i as f64 * 0.61803398875; // golden angle for spacing
        let r = radius * (i as f64) / (n as f64); // radius * i/n where i/n goes from 0 to 1
        rx.push(r * angle.cos());
        ry.push(r * angle.sin());

        rz.push(height * ((i as f64) / (n as f64) - 0.5)); // z from -height/2 to +height/2

        // Masses vary slightly but repeatably
        masses.push(mass_base + (i % 10) as f64 * 0.1);
    }

    (masses, rx, ry, rz)
}

fn bench_gravity_methods(c: &mut Criterion, gravity_methods: &[GravityMethod], n_values: &[usize]) {
    let args = Args::default();
    let parameters = Parameters::new(
        args.time_step,
        args.num_steps,
        args.g_constant,
        args.softening_factor,
        args.theta,
        args.progress,
    );

    let group_name = gravity_methods
        .iter()
        .map(|s| format!("{:?}", s))
        .collect::<Vec<_>>()
        .join("-vs-");

    let mut group = c.benchmark_group(group_name);
    for gravity_method in gravity_methods {
        for &n in n_values {
            let (masses, rx, ry, rz) = generate_distributed_bodies_positions(n);
            let (ax, ay, az) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", gravity_method), n),
                &n,
                |b, &n| {
                    b.iter_batched_ref(
                        || {
                            (
                                gravity_method.create(&parameters, n),
                                masses.clone(),
                                rx.clone(),
                                ry.clone(),
                                rz.clone(),
                                ax.clone(),
                                ay.clone(),
                                az.clone(),
                            )
                        },
                        |tuple| {
                            let (gravity_box, masses, rx, ry, rz, ax, ay, az) = tuple;
                            gravity_box
                                .as_mut()
                                .calculate_accelerations(masses, rx, ry, rz, ax, ay, az);
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
}

#[allow(dead_code)]
fn bench_newton_acceleration(c: &mut Criterion) {
    let gravity_methods = [GravityMethod::Newton];
    let n_values = [2, 5, 10, 25, 50, 75, 100, 150, 200];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_newton_parallel_acceleration(c: &mut Criterion) {
    let gravity_methods = [GravityMethod::NewtonParallel];
    let n_values = [2, 5, 10, 25, 50, 75, 100, 150, 200];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_newton_vs_parallel_acceleration(c: &mut Criterion) {
    let gravity_methods = [GravityMethod::Newton, GravityMethod::NewtonParallel];
    let n_values = [
        3, 5, 10, 15, 20, 25, 50, 75, 100, 150, 200, 250, 300, 400, 500,
    ];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_barnes_hut_acceleration(c: &mut Criterion) {
    let gravity_methods = [GravityMethod::BarnesHut];
    let n_values = [100, 200, 300, 400, 500, 750, 1000, 1500, 2000];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_newton_parallel_vs_barnes_hut_acceleration(c: &mut Criterion) {
    let gravity_methods = [GravityMethod::NewtonParallel, GravityMethod::BarnesHut];
    let n_values = [
        10, 25, 50, 100, 200, 300, 400, 500, 750, 1000, 1250, 1500, 2000, 2500, 3000,
    ];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

criterion_group!(
    benches, // macro-generated "benches" method
    // bench_newton_acceleration,
    // bench_newton_parallel_acceleration,
    bench_newton_vs_parallel_acceleration,
    // bench_barnes_hut_acceleration,
    bench_newton_parallel_vs_barnes_hut_acceleration,
);
criterion_main!(benches);
