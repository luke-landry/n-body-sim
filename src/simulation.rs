use crate::gravity::Gravity;
use crate::input::InitialCondition;
use crate::integrators::Integrator;
use std::fmt;

const ZERO_VECTOR: [f64; 2] = [0.0, 0.0];

pub struct Body {
    pub id: usize,
    pub mass: f64,
    pub position: [f64; 2],
    pub velocity: [f64; 2],
}

impl Body {
    pub fn new(id: usize, mass: f64, position: [f64; 2], velocity: [f64; 2]) -> Self {
        Body {
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
}

impl Parameters {
    pub fn new(
        time_step: f64,
        num_steps: usize,
        g_constant: f64,
        softening_factor: f64,
        theta: f64,
    ) -> Self {
        Parameters {
            time_step,
            num_steps,
            g_constant,
            softening_factor,
            theta,
        }
    }
}

pub struct BodySnapshot {
    time: f64,
    id: usize,
    pos_x: f64,
    pos_y: f64,
}

impl BodySnapshot {
    fn create(body: &Body, time: f64) -> Self {
        BodySnapshot {
            time,
            id: body.id,
            pos_x: body.position[0],
            pos_y: body.position[1],
        }
    }
}

impl fmt::Display for BodySnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Time: {:>10.4}s | Body {:>3} | Position: ({:>12.6}, {:>12.6})",
            self.time, self.id, self.pos_x, self.pos_y
        )
    }
}

pub type Data = Vec<BodySnapshot>;

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

    pub fn run(&mut self) -> Data {
        // Precompute total results size to avoid memory reallocations
        let num_results = self.bodies.len() * (self.parameters.num_steps + 1);
        let mut data: Data = Vec::with_capacity(num_results);

        let mut record_state = |bodies: &[Body], time: f64| {
            data.extend(bodies.iter().map(|body| BodySnapshot::create(body, time)));
        };

        let mut time = 0.0;
        for _ in 0..self.parameters.num_steps {
            record_state(&self.bodies, time);
            self.integrator
                .step(&mut self.bodies, self.parameters.time_step);
            time += self.parameters.time_step;
        }

        // Capture final state
        record_state(&self.bodies, time);

        data
    }
}
