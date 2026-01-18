use crate::gravity::Gravity;
use crate::input::InitialCondition;
use crate::integrators::Integrator;

const ZERO_VECTOR: [f64; 2] = [0.0, 0.0];

pub struct Body {
    pub name: String,
    pub mass: f64,
    pub position: [f64; 2],
    pub velocity: [f64; 2],
    pub acceleration: [f64; 2],
}

impl Body {
    pub fn new(name: String, mass: f64, position: [f64; 2], velocity: [f64; 2]) -> Self {
        Body {
            name,
            mass,
            position,
            velocity,
            acceleration: ZERO_VECTOR,
        }
    }
}

impl From<InitialCondition> for Body {
    fn from(value: InitialCondition) -> Self {
        Body {
            name: value.name,
            mass: value.mass,
            position: [value.pos_x, value.pos_y],
            velocity: [value.vel_x, value.vel_y],
            acceleration: ZERO_VECTOR,
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
    name: String,
    pos_x: f64,
    pos_y: f64,
}

pub type Data = Vec<BodySnapshot>;

pub struct Simulator {
    bodies: Vec<Body>,
    parameters: Parameters,
    gravity: Box<dyn Gravity>,
    integrator: Box<dyn Integrator>,
    data: Data,
    step_count: usize,
}

impl Simulator {
    pub fn new(
        bodies: Vec<Body>,
        parameters: Parameters,
        gravity: Box<dyn Gravity>,
        integrator: Box<dyn Integrator>,
    ) -> Self {
        // The number of calculated results will be # of bodies * # of steps
        let num_results = bodies.len() * parameters.num_steps;

        Simulator {
            bodies,
            parameters,
            gravity,
            integrator,
            data: Vec::with_capacity(num_results),
            step_count: 0,
        }
    }

    pub fn run(&mut self) -> Data {
        vec![]
    }
}
