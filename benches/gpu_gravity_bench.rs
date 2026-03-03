mod common_bench;
use std::sync::OnceLock;

use common_bench::generate_distributed_bodies_positions;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use n_body_sim::gpu::CudaManager;
use n_body_sim::gpu::device_bodies::DeviceBodies;
use n_body_sim::simulation::Parameters;
use n_body_sim::{
    args::{
        Args,
        GravityMethod::{self, NewtonParallel},
    },
    body::Bodies,
};

// Global GPU manager instance initialized once for all benchmarks that need it
static GPU: OnceLock<CudaManager> = OnceLock::new();

/*
  Criterion benchmarks for n-body simulation gravity methods
  comparing CPU vs GPU implementations.

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
    bench_gpu_newton_parallel_acceleration,
);
criterion_main!(benches);

#[allow(dead_code)]
fn bench_gpu_newton_parallel_acceleration(c: &mut Criterion) {
    let gravity_methods = [NewtonParallel];
    let n_values = [1000, 2000, 3000, 4000, 5000, 7500, 10000, 15000, 20000];
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

    // Bench CPU versions
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

    // Bench GPU versions
    let gpu = GPU.get_or_init(|| {
        let manager = CudaManager::new().expect("Failed to initialize CUDA");
        manager
            .gpu_init_check()
            .expect("GPU initialization check failed");
        manager
    });
    for gravity_method in gravity_methods {
        for &n in n_values {
            let (masses, rx, ry, rz) = generate_distributed_bodies_positions(n);
            let (vx, vy, vz) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
            let bodies = Bodies::new(masses, rx, ry, rz, vx, vy, vz);
            group.bench_with_input(
                BenchmarkId::new(format!("GPU-{:?}", gravity_method), n),
                &n,
                |b, &_n| {
                    b.iter_batched_ref(
                        || {
                            (
                                gravity_method.gpu_create(&parameters),
                                DeviceBodies::new(gpu, &bodies)
                                    .expect("Failed to create DeviceBodies"),
                            )
                        },
                        |(gpu_gravity, device_bodies)| {
                            gpu_gravity
                                .calculate_accelerations(gpu, device_bodies)
                                .expect("Failed to calculate accelerations on GPU");
                            gpu.stream
                                .synchronize()
                                .expect("Failed to synchronize GPU after acceleration calculation");
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.finish();
}
