mod constants;
mod gravity;
mod input;
mod integrators;
mod output;
mod simulation;

use clap::{Parser, ValueEnum};
use gravity::{Gravity, newton::NewtonGravity};
use integrators::{Integrator, euler::EulerIntegrator};
use simulation::Simulator;
use std::{error::Error, path::PathBuf};

use crate::{
    gravity::barnes_hut::BarnesHutGravity, integrators::velocity_verlet::VelocityVerletIntegrator,
    simulation::Parameters,
};

#[derive(Parser)]
struct Args {
    /// Path to CSV file containing initial conditions for each body
    #[arg(short, long)]
    initial_conditions_path: PathBuf,

    /// Path to CSV file to save simulation output data (prints to stdout if none provided)
    #[arg(short, long)]
    output_data_path: Option<PathBuf>,

    /// The gravitional constant to use in gravitational force calculations
    #[arg(short, long, default_value_t=constants::DEFAULT_G)]
    g_constant: f64,

    /// Simulation time step in seconds
    #[arg(short, long, default_value_t = constants::DEFAULT_TIMESTEP)]
    time_step: f64,

    /// Number of time steps to simulate
    #[arg(short, long, default_value_t = constants::DEFAULT_NUM_STEPS)]
    num_steps: usize,

    /// The softening factor to avoid numerical instability as distances approach zero
    #[arg(long, default_value_t = constants::DEFAULT_SOFTENING_FACTOR)]
    softening_factor: f64,

    /// Theta value for Barnes-Hut gravity calculation method
    #[arg(long, default_value_t = constants::DEFAULT_THETA)]
    theta: f64,

    /// Gravity force calculation method
    #[arg(long, default_value = constants::DEFAULT_GRAVITY)]
    gravity: GravityMethod,

    /// Integrator for computing next-step state
    #[arg(long, default_value = constants::DEFAULT_INTEGRATOR)]
    integrator: IntegratorMethod,
}

#[derive(Clone, ValueEnum)]
enum GravityMethod {
    Newton,
    BarnesHut,
}

impl GravityMethod {
    pub fn create(&self, parameters: &Parameters) -> Box<dyn Gravity> {
        match self {
            GravityMethod::Newton => Box::new(NewtonGravity::new(
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
enum IntegratorMethod {
    Euler,
    VelocityVerlet,
}

impl IntegratorMethod {
    pub fn create(&self, gravity: Box<dyn Gravity>) -> Box<dyn Integrator> {
        match self {
            IntegratorMethod::Euler => Box::new(EulerIntegrator::new(gravity)),
            IntegratorMethod::VelocityVerlet => Box::new(VelocityVerletIntegrator::new(gravity)),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("n-body-sim");

    let args = Args::parse();

    let bodies = input::load_bodies(&args.initial_conditions_path)?;

    let parameters = simulation::Parameters::new(
        args.time_step,
        args.num_steps,
        args.g_constant,
        args.softening_factor,
        args.theta,
    );

    let gravity = args.gravity.create(&parameters);

    let integrator = args.integrator.create(gravity);

    let mut simulator = Simulator::new(bodies, parameters, integrator);

    let data = simulator.run();

    match args.output_data_path {
        Some(path) => {
            output::save_to_csv(&path, data)?;
        }
        None => {
            output::print_data(data);
        }
    }

    Ok(())
}
