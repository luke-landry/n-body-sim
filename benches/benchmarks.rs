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

// Benchmark for Newton gravity method
fn bench_newton(c: &mut Criterion) {
    const N_VALUES: [usize; 8] = [2, 3, 5, 10, 25, 50, 100, 250];
    const INTEGRATOR_METHODS: [IntegratorMethod; 3] = [
        IntegratorMethod::Euler,
        IntegratorMethod::VelocityVerlet,
        IntegratorMethod::RungeKutta,
    ];

    let args = Args::default();
    let mut group = c.benchmark_group("Newton");
    for &n in &N_VALUES {
        for &integrator_method in &INTEGRATOR_METHODS {
            group.bench_with_input(
                BenchmarkId::new(
                    format!("N={}", n),
                    format!("Integrator={:?}", integrator_method),
                ),
                &(n, integrator_method),
                |b, &(n, integrator_method)| {
                    b.iter_batched_ref(
                        || create_simulation(n, GravityMethod::Newton, integrator_method, &args),
                        |sim| sim.run(),
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
}

criterion_group!(benches, bench_newton);
criterion_main!(benches);
