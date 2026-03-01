mod common_bench;
use common_bench::generate_distributed_bodies_positions;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use n_body_sim::args::{Args, GravityMethod::BarnesHut};
use n_body_sim::simulation::Parameters;

/*
  Criterion benchmarks for n-body simulation barnes-hut method

  To configure these benchmarks, change the theta_values and
  n_values arrays in the individual benchmark functions below.

  The criterion_group! macro is used to specify which benchmark functions to run.
  Uncomment the desired benchmark functions to include them in the benchmark suite.

  To run these benchmarks, run 'cargo bench --bench barnes_hut_bench' in the project root.
*/

criterion_group!(
    benches, // macro-generated "benches" method
    /*
        Uncomment the benchmark functions below to run them
    */
    bench_barnes_hut_theta,
);
criterion_main!(benches);

#[allow(dead_code)]
fn bench_barnes_hut_theta(c: &mut Criterion) {
    let theta_values = [0.0, 0.1, 0.25, 0.5, 0.75, 1.0, 1.25];
    let n_values = [1000, 1500, 2000, 2500, 3000];
    bench_barnes_hut_method(c, &theta_values, &n_values);
}

fn bench_barnes_hut_method(c: &mut Criterion, theta_values: &[f64], n_values: &[usize]) {
    let args = Args::default();

    let mut group = c.benchmark_group("BarnesHut-theta");
    for &theta_value in theta_values {
        let parameters = Parameters::new(
            args.time_step,
            args.num_steps,
            args.g_constant,
            args.softening_factor,
            theta_value,
            args.progress,
        );
        for &n in n_values {
            let (masses, rx, ry, rz) = generate_distributed_bodies_positions(n);
            let (ax, ay, az) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
            group.bench_with_input(
                BenchmarkId::new(format!("theta={:?}", theta_value), n),
                &n,
                |b, &n| {
                    b.iter_batched_ref(
                        || {
                            (
                                BarnesHut.create(&parameters, n),
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
