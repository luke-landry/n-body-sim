pub mod cpu_simulation;
pub mod gpu_simulation;

use crate::output::SimulationSnapshot;
use std::error::Error;
pub struct SimulationParameters {
    /// Time step size in seconds
    pub time_step: f64,

    /// Number of time steps to simulate
    pub num_steps: usize,

    /// The gravitational constant to use for gravitational force calculations
    pub g_constant: f64,

    /// The softening factor used to prevent numerical instability
    /// when the distance between two bodies approaches zero
    pub softening_factor: f64,

    /// The theta value used in Barnes-Hut gravity force calculations
    pub theta: f64,

    /// Whether to output step progress
    pub progress: bool,
}

impl SimulationParameters {
    pub fn new(
        time_step: f64,
        num_steps: usize,
        g_constant: f64,
        softening_factor: f64,
        theta: f64,
        progress: bool,
    ) -> Self {
        SimulationParameters {
            time_step,
            num_steps,
            g_constant,
            softening_factor,
            theta,
            progress,
        }
    }
}

pub trait Simulation: Send {
    fn step(&mut self) -> Result<(), Box<dyn Error>>;
    fn snapshot(&self) -> Result<SimulationSnapshot, Box<dyn Error>>;
}
