use crate::integrators::Integrator;
use crate::output::BodySnapshot;
use glam::DVec3;

#[derive(Clone)]
pub struct Body {
    pub id: usize,
    pub mass: f64,
    pub position: DVec3,
    pub velocity: DVec3,
}

impl Body {
    pub fn new(id: usize, mass: f64, position: DVec3, velocity: DVec3) -> Self {
        Self {
            id,
            mass,
            position,
            velocity,
        }
    }
}

pub struct Parameters {
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

impl Parameters {
    pub fn new(
        time_step: f64,
        num_steps: usize,
        g_constant: f64,
        softening_factor: f64,
        theta: f64,
        progress: bool,
    ) -> Self {
        Parameters {
            time_step,
            num_steps,
            g_constant,
            softening_factor,
            theta,
            progress,
        }
    }
}

pub struct Simulator {
    bodies: Vec<Body>,
    parameters: Parameters,
    integrator: Box<dyn Integrator>,
}

impl Simulator {
    pub fn new(bodies: Vec<Body>, parameters: Parameters, integrator: Box<dyn Integrator>) -> Self {
        Simulator {
            bodies,
            parameters,
            integrator,
        }
    }

    pub fn run(&mut self) -> Vec<BodySnapshot> {
        // Precompute total results size to avoid memory reallocations
        let num_results = self.bodies.len() * (self.parameters.num_steps + 1);
        let one_percent_steps = (self.parameters.num_steps / 100).max(1);
        let mut data: Vec<BodySnapshot> = Vec::with_capacity(num_results);
        let mut time = 0.0;
        let mut record_state = |bodies: &[Body], time: f64| {
            data.extend(bodies.iter().map(|body| BodySnapshot::create(body, time)));
        };

        for step in 0..self.parameters.num_steps {
            record_state(&self.bodies, time);
            self.integrator.step(&mut self.bodies);
            time += self.parameters.time_step;

            if self.parameters.progress && step % one_percent_steps == 0 {
                println!("{}", (step * 100) / self.parameters.num_steps);
            }
        }

        // Capture final state
        record_state(&self.bodies, time);

        data
    }
}
