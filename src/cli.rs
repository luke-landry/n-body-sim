use crate::constants;
use crate::gravity::{
    Gravity, barnes_hut::BarnesHutGravity, newton::NewtonGravity,
    newton_parallel::NewtonParallelGravity,
};
use crate::integrators::{
    Integrator, euler::EulerIntegrator, velocity_verlet::VelocityVerletIntegrator,
};
use crate::simulation::Parameters;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    /// Path to CSV file containing initial conditions for each body
    #[arg(short, long, default_value = constants::DEFAULT_INITIAL_CONDITIONS_PATH)]
    pub initial_conditions_path: PathBuf,

    /// Path to csv or nbody file to save simulation output data
    #[arg(short, long, default_value = constants::DEFAULT_OUTPUT_PATH)]
    pub output_data_path: PathBuf,

    /// The gravitational constant to use in force calculations
    #[arg(short, long, default_value_t=constants::DEFAULT_G)]
    pub g_constant: f64,

    /// Simulation time step in seconds
    #[arg(short, long, default_value_t = constants::DEFAULT_TIMESTEP)]
    pub time_step: f64,

    /// Number of time steps to simulate
    #[arg(short, long, default_value_t = constants::DEFAULT_NUM_STEPS)]
    pub num_steps: usize,

    /// The softening factor to avoid numerical instability as distances approach zero
    #[arg(long, default_value_t = constants::DEFAULT_SOFTENING_FACTOR)]
    pub softening_factor: f64,

    /// Theta value for Barnes-Hut gravity calculation method
    #[arg(long, default_value_t = constants::DEFAULT_THETA)]
    pub theta: f64,

    /// Gravity force calculation method
    #[arg(long, default_value = constants::DEFAULT_GRAVITY)]
    pub gravity: GravityMethod,

    /// Integrator for computing next-step state
    #[arg(long, default_value = constants::DEFAULT_INTEGRATOR)]
    pub integrator: IntegratorMethod,

    /// Flag to enable percent progress output to stdout
    #[arg(long)]
    pub progress: bool,

    /// Flag to run benchmarks instead of a simulation.
    /// When set, the file paths and gravity and integrator parameters are ignored
    #[arg(long)]
    pub benchmark: bool,

    /// Flag to specify the N values to use for benchmarks (comma-separated list, e.g. "10,50,250")
    #[arg(long, value_delimiter = ',', default_value = constants::DEFAULT_BENCHMARK_N_VALUES)]
    pub benchmark_n_values: Vec<usize>,

    /// Comma-separated list of gravity methods to use for benchmarks
    #[arg(long, value_delimiter = ',', default_value = constants::DEFAULT_BENCHMARK_GRAVITY_METHODS)]
    pub benchmark_gravity_methods: Vec<GravityMethod>,

    /// Comma-separated list of integrator methods to use for benchmarks
    #[arg(long, value_delimiter = ',', default_value = constants::DEFAULT_BENCHMARK_INTEGRATOR_METHODS)]
    pub benchmark_integrator_methods: Vec<IntegratorMethod>,

    /// Number of benchmark runs per N-gravity-integrator combination
    #[arg(long, default_value_t = constants::DEFAULT_BENCHMARK_NUM_RUNS)]
    pub benchmark_num_runs: usize,

    /// Path to CSV file to save benchmark results
    #[arg(long, default_value = constants::DEFAULT_BENCHMARK_OUTPUT_PATH)]
    pub benchmark_output_path: PathBuf,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum GravityMethod {
    Newton,
    NewtonParallel,
    BarnesHut,
}

impl GravityMethod {
    pub fn create(&self, parameters: &Parameters, n: usize) -> Box<dyn Gravity> {
        match self {
            GravityMethod::Newton => Box::new(NewtonGravity::new(
                parameters.g_constant,
                parameters.softening_factor,
            )),
            GravityMethod::NewtonParallel => Box::new(NewtonParallelGravity::new(
                parameters.g_constant,
                parameters.softening_factor,
            )),
            GravityMethod::BarnesHut => Box::new(BarnesHutGravity::new(
                parameters.g_constant,
                parameters.softening_factor,
                parameters.theta,
                n,
            )),
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum IntegratorMethod {
    Euler,
    VelocityVerlet,
    RungeKutta,
}

impl IntegratorMethod {
    pub fn create(
        &self,
        gravity: Box<dyn Gravity>,
        time_step: f64,
        num_bodies: usize,
    ) -> Box<dyn Integrator> {
        match self {
            IntegratorMethod::Euler => {
                Box::new(EulerIntegrator::new(gravity, time_step, num_bodies))
            }
            IntegratorMethod::VelocityVerlet => Box::new(VelocityVerletIntegrator::new(
                gravity, time_step, num_bodies,
            )),
            IntegratorMethod::RungeKutta => {
                Box::new(crate::integrators::runge_kutta::RungeKuttaIntegrator::new(
                    gravity, time_step, num_bodies,
                ))
            }
        }
    }
}
