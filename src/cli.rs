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
    #[arg(short, long)]
    pub initial_conditions_path: PathBuf,

    /// Path to CSV file to save simulation output data (prints to stdout if none provided)
    #[arg(short, long)]
    pub output_data_path: Option<PathBuf>,

    /// The gravitional constant to use in gravitational force calculations
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
}

#[derive(Clone, ValueEnum)]
pub enum GravityMethod {
    Newton,
    NewtonParallel,
    BarnesHut,
}

impl GravityMethod {
    pub fn create(&self, parameters: &Parameters) -> Box<dyn Gravity> {
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
            )),
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum IntegratorMethod {
    Euler,
    VelocityVerlet,
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
        }
    }
}
