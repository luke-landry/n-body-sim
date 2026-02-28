mod common_bench;
use common_bench::generate_distributed_bodies_positions;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use n_body_sim::args::{
    Args, GravityMethod,
    GravityMethod::Newton,
    IntegratorMethod,
    IntegratorMethod::{Euler, RungeKutta, VelocityVerlet},
};
use n_body_sim::simulation::{Bodies, Parameters};

/*
  Criterion benchmarks for n-body simulation integrators

  To configure these benchmarks, change the integrator_methods,
  gravity_methods, and n_values arrays in the individual benchmark functions below.

  The criterion_group! macro is used to specify which benchmark functions to run.
  Uncomment the desired benchmark functions to include them in the benchmark suite.

  To run these benchmarks, run 'cargo bench --bench integrator_bench' in the project root.
*/

criterion_group!(
    benches, // macro-generated "benches" method
    /*
        Uncomment the benchmark functions below to run them
    */
    // bench_euler_step,
    // bench_velocity_verlet_step,
    // bench_runge_kutta_step,
    bench_all_methods_step,
);
criterion_main!(benches);

#[allow(dead_code)]
fn bench_euler_step(c: &mut Criterion) {
    let integrator_methods = [Euler];
    let gravity_methods = [Newton];
    let n_values = [20, 40, 60, 80, 100];
    bench_integrator_methods(c, &integrator_methods, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_velocity_verlet_step(c: &mut Criterion) {
    let integrator_methods = [VelocityVerlet];
    let gravity_methods = [Newton];
    let n_values = [20, 40, 60, 80, 100];
    bench_integrator_methods(c, &integrator_methods, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_runge_kutta_step(c: &mut Criterion) {
    let integrator_methods = [RungeKutta];
    let gravity_methods = [Newton];
    let n_values = [20, 40, 60, 80, 100];
    bench_integrator_methods(c, &integrator_methods, &gravity_methods, &n_values);
}

#[allow(dead_code)]
fn bench_all_methods_step(c: &mut Criterion) {
    let integrator_methods = [Euler, VelocityVerlet, RungeKutta];
    let gravity_methods = [Newton];
    let n_values = [20, 40, 60, 80, 100];
    bench_integrator_methods(c, &integrator_methods, &gravity_methods, &n_values);
}

fn bench_integrator_methods(
    c: &mut Criterion,
    integrator_methods: &[IntegratorMethod],
    gravity_methods: &[GravityMethod],
    n_values: &[usize],
) {
    let args = Args::default();
    let parameters = Parameters::new(
        args.time_step,
        args.num_steps,
        args.g_constant,
        args.softening_factor,
        args.theta,
        args.progress,
    );

    let integrator_names = integrator_methods
        .iter()
        .map(|s| format!("{:?}", s))
        .collect::<Vec<_>>()
        .join("-vs-");

    let gravity_names = gravity_methods
        .iter()
        .map(|s| format!("{:?}", s))
        .collect::<Vec<_>>()
        .join("-and-");

    let group_name = format!("{}-using-{}", integrator_names, gravity_names);

    let mut group = c.benchmark_group(group_name);
    for integrator_method in integrator_methods {
        for gravity_method in gravity_methods {
            for &n in n_values {
                let (masses, rx, ry, rz) = generate_distributed_bodies_positions(n);
                let (vx, vy, vz) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
                group.bench_with_input(
                    BenchmarkId::new(format!("{:?}/{:?}", integrator_method, gravity_method), n),
                    &n,
                    |b, &n| {
                        b.iter_batched_ref(
                            || {
                                (
                                    integrator_method.create(
                                        gravity_method.create(&parameters, n),
                                        parameters.time_step,
                                        n,
                                    ),
                                    Bodies::new(
                                        masses.clone(),
                                        rx.clone(),
                                        ry.clone(),
                                        rz.clone(),
                                        vx.clone(),
                                        vy.clone(),
                                        vz.clone(),
                                    ),
                                )
                            },
                            |(integrator, bodies)| {
                                integrator.step(bodies);
                            },
                            BatchSize::SmallInput,
                        );
                    },
                );
            }
        }
    }
    group.finish();
}
