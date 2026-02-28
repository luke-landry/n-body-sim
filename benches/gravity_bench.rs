mod common_bench;
use common_bench::generate_distributed_bodies_positions;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use n_body_sim::args::{
    Args, GravityMethod,
    GravityMethod::{BarnesHut, Newton, NewtonParallel},
};
use n_body_sim::simulation::Parameters;

/*
  Criterion benchmarks for n-body simulation gravity methods

  To configure these benchmarks, change the gravity_methods and
  n_values arrays in the individual benchmark functions below.

  The criterion_group! macro is used to specify which benchmark functions to run.
  Uncomment the desired benchmark functions to include them in the benchmark suite.

  To run these benchmarks, run 'cargo bench --bench gravity_bench' in the project root.
*/

criterion_group!(
    benches, // macro-generated "benches" method
    /*
        Uncomment the benchmark functions below to run them
    */
    // bench_newton_acceleration,
    // bench_newton_parallel_acceleration,
    // bench_barnes_hut_acceleration,
    bench_newton_vs_parallel_acceleration,
    bench_newton_parallel_vs_barnes_hut_acceleration,
    bench_all_methods_acceleration,
);
criterion_main!(benches);

#[allow(dead_code)]
fn bench_newton_acceleration(c: &mut Criterion) {
    let gravity_methods = [Newton];
    let n_values = [2, 5, 10, 25, 50, 75, 100, 150, 200];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_newton_parallel_acceleration(c: &mut Criterion) {
    let gravity_methods = [NewtonParallel];
    let n_values = [2, 5, 10, 25, 50, 75, 100, 150, 200];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_barnes_hut_acceleration(c: &mut Criterion) {
    let gravity_methods = [BarnesHut];
    let n_values = [100, 200, 300, 400, 500, 750, 1000, 1500, 2000];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_newton_vs_parallel_acceleration(c: &mut Criterion) {
    let gravity_methods = [Newton, NewtonParallel];
    let n_values = [3, 5, 10, 15, 20, 25, 50, 75, 100, 150, 200, 250];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_newton_parallel_vs_barnes_hut_acceleration(c: &mut Criterion) {
    let gravity_methods = [NewtonParallel, BarnesHut];
    let n_values = [
        100, 200, 300, 400, 500, 750, 1000, 1250, 1500, 1750, 2000, 2500,
    ];
    bench_gravity_methods(c, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_all_methods_acceleration(c: &mut Criterion) {
    let gravity_methods = [Newton, NewtonParallel, BarnesHut];
    let n_values = [
        50, 100, 250, 500, 750, 1000, 1250, 1500, 1750, 2000, 2500, 3000,
    ];
    bench_gravity_methods(c, &gravity_methods, &n_values);
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
                        |(gravity, masses, rx, ry, rz, ax, ay, az)| {
                            gravity.calculate_accelerations(masses, rx, ry, rz, ax, ay, az);
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}
