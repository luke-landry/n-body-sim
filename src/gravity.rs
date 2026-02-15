use glam::DVec3;

use crate::simulation::Bodies;

pub mod barnes_hut;
pub mod newton;
pub mod newton_parallel;

/// SoA representation of accelerations
pub struct Accelerations {
    pub ax: Vec<f64>,
    pub ay: Vec<f64>,
    pub az: Vec<f64>,
}

impl Accelerations {
    pub fn zero(&mut self) {
        self.ax.fill(0.0);
        self.ay.fill(0.0);
        self.az.fill(0.0);
    }
}

pub trait Gravity {
    /// Writes accelerations into output parameter instead of returning a vec to avoid
    /// heap allocation on every step by allowing buffer reuse in the main simulation loop.
    /// The accelerations buffer must be zeroed before each call to this function.
    fn calculate_accelerations(&self, bodies: &Bodies, accelerations: &mut Accelerations);
}
