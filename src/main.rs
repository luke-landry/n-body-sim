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

#[derive(Parser)]
struct Args {
    /// Path to CSV file containing initial conditions for each body
    #[arg(short, long)]
    initial_conditions_path: PathBuf,

    /// Path to CSV file to save simulation output data
    #[arg(short, long, default_value = "n-body-sim-output.csv")]
    output_data_path: PathBuf,

    /// The gravitional constant to use in gravitational force calculations
    #[arg(long, default_value_t=constants::G)]
    g_constant: f64,

    /// Simulation time step in seconds
    #[arg(long, default_value_t = constants::DEFAULT_TIMESTEP)]
    time_step: f64,

    /// Number of time steps to simulate
    #[arg(long, default_value_t = constants::DEFAULT_NUM_STEPS)]
    num_steps: usize,

    /// The softening factor to avoid numerical instability as distances approach zero
    #[arg(long, default_value_t = constants::DEFAULT_SOFTENING_FACTOR)]
    softening_factor: f64,

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
}

impl GravityMethod {
    pub fn create(&self) -> Box<dyn Gravity> {
        match self {
            GravityMethod::Newton => Box::new(NewtonGravity),
        }
    }
}

#[derive(Clone, ValueEnum)]
enum IntegratorMethod {
    Euler,
}

impl IntegratorMethod {
    pub fn create(&self) -> Box<dyn Integrator> {
        match self {
            IntegratorMethod::Euler => Box::new(EulerIntegrator),
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
    );

    let gravity = args.gravity.create();

    let integrator = args.integrator.create();

    let mut simulator = Simulator::new(bodies, parameters, gravity, integrator);

    let data = simulator.run();

    output::save_to_csv(&args.output_data_path, data)?;

    Ok(())
}
