mod common_bench;
use std::sync::OnceLock;

use common_bench::generate_distributed_bodies_positions;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use n_body_sim::body::Bodies;
use n_body_sim::gpu::device_bodies::DeviceBodies;
use n_body_sim::simulation::Parameters;
use n_body_sim::{
    args::{
        Args,
        GravityMethod::{self, NewtonParallel},
        IntegratorMethod::{self, Euler},
    },
    gpu::CudaManager,
};

/*
  Criterion benchmarks for n-body simulation integrators

  To configure these benchmarks, change the integrator_methods,
  gravity_methods, and n_values arrays in the individual benchmark functions below.

  The criterion_group! macro is used to specify which benchmark functions to run.
  Uncomment the desired benchmark functions to include them in the benchmark suite.

  To run these benchmarks, run 'cargo bench --bench integrator_bench' in the project root.
*/

// Global GPU manager instance initialized once for all benchmarks that need it
static GPU: OnceLock<CudaManager> = OnceLock::new();

criterion_group!(
    benches, // macro-generated "benches" method
    /*
        Uncomment the benchmark functions below to run them
    */
    bench_euler_step,
);
criterion_main!(benches);

#[allow(dead_code)]
fn bench_euler_step(c: &mut Criterion) {
    let integrator_methods = [Euler];
    let gravity_methods = [NewtonParallel];
    let n_values = [
        1000, 2000, 3000, 4000, 5000, 7500, 10000, 15000, 20000, 25000,
    ];
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

    // bench CPU versions
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

    // bench GPU versions
    let gpu = GPU.get_or_init(|| {
        let manager = CudaManager::new().expect("Failed to initialize CUDA");
        manager
            .gpu_init_check()
            .expect("GPU initialization check failed");
        manager
    });
    for integrator_method in integrator_methods {
        for gravity_method in gravity_methods {
            for &n in n_values {
                let (masses, rx, ry, rz) = generate_distributed_bodies_positions(n);
                let (vx, vy, vz) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
                let bodies = Bodies::new(masses, rx, ry, rz, vx, vy, vz);
                group.bench_with_input(
                    BenchmarkId::new(
                        format!("GPU-{:?}/GPU-{:?}", integrator_method, gravity_method),
                        n,
                    ),
                    &n,
                    |b, &_n| {
                        b.iter_batched_ref(
                            || {
                                (
                                    integrator_method.gpu_create(parameters.time_step),
                                    gravity_method.gpu_create(&parameters),
                                    DeviceBodies::new(gpu, &bodies)
                                        .expect("Failed to create DeviceBodies"),
                                )
                            },
                            |(gpu_integrator, gpu_gravity, device_bodies)| {
                                gpu_integrator
                                    .step(gpu, device_bodies, gpu_gravity.as_ref())
                                    .expect("Failed to perform integration step on GPU");
                                gpu.stream
                                    .synchronize()
                                    .expect("Failed to synchronize GPU after integration step");
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
