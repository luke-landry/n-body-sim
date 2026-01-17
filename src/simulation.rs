use serde::Deserialize;

const ZERO_VECTOR: [f64; 2] = [0.0, 0.0];

#[derive(Deserialize, Debug)]
pub struct InitialCondition {
    name: String,
    mass: f64,
    pos_x: f64,
    pos_y: f64,
    vel_x: f64,
    vel_y: f64,
}

pub struct Body {
    name: String,
    mass: f64,
    position: [f64; 2],
    velocity: [f64; 2],
    acceleration: [f64; 2],
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
    time_step: f64,

    /// Number of time steps to simulate
    num_steps: usize,

    /// The gravitational constant to use for gravitational force calculations
    g_constant: f64,

    /// The softening factor used to prevent numerical instability
    /// when the distance between two bodies approaches zero
    softening_factor: f64,
}

impl Parameters {
    pub fn new(time_step: f64, num_steps: usize, g_constant: f64, softening_factor: f64) -> Self {
        Parameters {
            time_step,
            num_steps,
            g_constant,
            softening_factor,
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

pub fn run(bodies: Vec<Body>, parameters: Parameters) -> Data {
    vec![]
}
