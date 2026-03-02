use crate::constants;
use crate::gpu::gravity::GpuGravity;
use crate::gpu::gravity::newton_parallel::GpuNewtonParallelGravity;
use crate::gpu::integrators::GpuIntegrator;
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
use std::str::FromStr;

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

    /// Flag to enable GPU acceleration (currently supported only for Newton Parallel and Euler methods)
    #[arg(long)]
    pub gpu: bool,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            initial_conditions_path: PathBuf::from(constants::DEFAULT_INITIAL_CONDITIONS_PATH),
            output_data_path: PathBuf::from(constants::DEFAULT_OUTPUT_PATH),
            g_constant: constants::DEFAULT_G,
            time_step: constants::DEFAULT_TIMESTEP,
            num_steps: constants::DEFAULT_NUM_STEPS,
            softening_factor: constants::DEFAULT_SOFTENING_FACTOR,
            theta: constants::DEFAULT_THETA,
            gravity: constants::DEFAULT_GRAVITY.parse().unwrap(),
            integrator: constants::DEFAULT_INTEGRATOR.parse().unwrap(),
            progress: false,
            gpu: false,
        }
    }
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

    pub fn gpu_create(&self, parameters: &Parameters) -> Box<dyn GpuGravity> {
        match self {
            GravityMethod::NewtonParallel => Box::new(GpuNewtonParallelGravity::new(
                parameters.g_constant,
                parameters.softening_factor.powi(2), // eps2
            )),
            _ => unimplemented!(
                "GPU-accelerated version of {:?} gravity method is not implemented yet",
                self
            ),
        }
    }
}

impl FromStr for GravityMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "newton" => Ok(GravityMethod::Newton),
            "newton-parallel" => Ok(GravityMethod::NewtonParallel),
            "barnes-hut" => Ok(GravityMethod::BarnesHut),
            _ => Err(format!("Invalid gravity method: {}", s)),
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

    pub fn gpu_create(&self, time_step: f64) -> Box<dyn GpuIntegrator> {
        match self {
            IntegratorMethod::Euler => Box::new(
                crate::gpu::integrators::euler::GpuEulerIntegrator::new(time_step),
            ),
            _ => unimplemented!(
                "GPU-accelerated version of {:?} integrator method is not implemented yet",
                self
            ),
        }
    }
}

impl FromStr for IntegratorMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "euler" => Ok(IntegratorMethod::Euler),
            "velocity-verlet" => Ok(IntegratorMethod::VelocityVerlet),
            "runge-kutta" => Ok(IntegratorMethod::RungeKutta),
            _ => Err(format!("Invalid integrator method: {}", s)),
        }
    }
}
