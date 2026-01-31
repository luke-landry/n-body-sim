use crate::integrators::Integrator;
use crate::output::BodySnapshot;
use glam::DVec3;

pub struct Body {
    pub id: usize,
    pub mass: f64,
    pub position: DVec3,
    pub velocity: DVec3,
}

impl Body {
    pub fn new(
        id: usize,
        mass: f64,
        position: DVec3,
        velocity: DVec3,
    ) -> Self {
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
        let mut data: Vec<BodySnapshot> = Vec::with_capacity(num_results);

        // Accelerations is allocated at sim top level so that it does not
        // need to be heap allocated on every step
        let mut accelerations = vec![DVec3::ZERO; self.bodies.len()];

        let mut record_state = |bodies: &[Body], time: f64| {
            data.extend(bodies.iter().map(|body| BodySnapshot::create(body, time)));
        };

        let mut time = 0.0;
        for _ in 0..self.parameters.num_steps {
            record_state(&self.bodies, time);
            self.integrator
                .step(&mut self.bodies, self.parameters.time_step, &mut accelerations);
            time += self.parameters.time_step;
        }

        // Capture final state
        record_state(&self.bodies, time);

        data
    }
}
