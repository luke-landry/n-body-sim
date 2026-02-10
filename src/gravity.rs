use glam::DVec3;

use crate::simulation::Body;

pub mod barnes_hut;
pub mod newton;

pub trait Gravity {
    /// Writes accelerations into output parameter instead of returning a vec to avoid
    /// heap allocation on every step by allowing buffer reuse in the main simulation loop.
    /// The accelerations buffer must be zeroed before each call to this function.
    fn calculate_accelerations(&self, bodies: &[Body], accelerations: &mut [DVec3]);
}
