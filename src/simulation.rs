use crate::integrators::Integrator;
use crate::output::{BodiesSnapshot, BodySnapshot, flatten_bodies_snapshots};
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

/// SoA representation of bodies
pub struct Bodies {
    pub masses: Vec<f64>,
    pub pos_x: Vec<f64>,
    pub pos_y: Vec<f64>,
    pub pos_z: Vec<f64>,
    pub vel_x: Vec<f64>,
    pub vel_y: Vec<f64>,
    pub vel_z: Vec<f64>,
}

impl Bodies {
    pub fn len(&self) -> usize {
        self.masses.len()
    }

    /// Return tuple of references to the mass, 3D position, and 3D velocity components
    pub fn as_slices(&mut self) -> (&[f64], &[f64], &[f64], &[f64], &[f64], &[f64], &[f64]) {
        (
            &self.masses,
            &self.pos_x,
            &self.pos_y,
            &self.pos_z,
            &self.vel_x,
            &self.vel_y,
            &self.vel_z,
        )
    }
    /// Return tuple of mutable references to the mass, 3D position, and 3D velocity components
    pub fn as_slices_mut(
        &mut self,
    ) -> (
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
        &mut [f64],
    ) {
        (
            &mut self.masses,
            &mut self.pos_x,
            &mut self.pos_y,
            &mut self.pos_z,
            &mut self.vel_x,
            &mut self.vel_y,
            &mut self.vel_z,
        )
    }
}

impl From<&[Body]> for Bodies {
    fn from(bodies: &[Body]) -> Self {
        let mut masses = Vec::with_capacity(bodies.len());
        let mut pos_x = Vec::with_capacity(bodies.len());
        let mut pos_y = Vec::with_capacity(bodies.len());
        let mut pos_z = Vec::with_capacity(bodies.len());
        let mut vel_x = Vec::with_capacity(bodies.len());
        let mut vel_y = Vec::with_capacity(bodies.len());
        let mut vel_z = Vec::with_capacity(bodies.len());

        for body in bodies {
            masses.push(body.mass);
            pos_x.push(body.position.x);
            pos_y.push(body.position.y);
            pos_z.push(body.position.z);
            vel_x.push(body.velocity.x);
            vel_y.push(body.velocity.y);
            vel_z.push(body.velocity.z);
        }

        Bodies {
            masses,
            pos_x,
            pos_y,
            pos_z,
            vel_x,
            vel_y,
            vel_z,
        }
    }
}

impl Into<Vec<Body>> for Bodies {
    fn into(self) -> Vec<Body> {
        let mut bodies = Vec::with_capacity(self.masses.len());
        for i in 0..self.masses.len() {
            bodies.push(Body {
                id: i,
                mass: self.masses[i],
                position: DVec3::new(self.pos_x[i], self.pos_y[i], self.pos_z[i]),
                velocity: DVec3::new(self.vel_x[i], self.vel_y[i], self.vel_z[i]),
            });
        }
        bodies
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

        // Data is a AoSoA where we record a set of body snapshots at each time step,
        // which will be converted to a flat Vec<BodySnapshot> for output. This avoids
        // performing conversions from SoA to AoS at each step during the simulation.
        let mut data_soa: Vec<BodiesSnapshot> = Vec::with_capacity(num_results);

        let mut bodies_soa = Bodies::from(self.bodies.as_slice());

        let mut time = 0.0;
        for step in 0..self.parameters.num_steps {
            data_soa.push(BodiesSnapshot::from_state(&bodies_soa, time));
            self.integrator.step(&mut bodies_soa);
            time += self.parameters.time_step;

            if self.parameters.progress && step % one_percent_steps == 0 {
                println!("{}", (step * 100) / self.parameters.num_steps);
            }
        }

        // Capture final state
        data_soa.push(BodiesSnapshot::from_state(&bodies_soa, time));

        flatten_bodies_snapshots(&data_soa)
    }
}
