use crate::simulation::Body;

pub mod barnes_hut;
pub mod newton;

pub trait Gravity {
    fn calculate_accelerations(&self, bodies: &[Body]) -> Vec<[f64; 2]>;
}
