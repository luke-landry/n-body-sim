pub mod barnes_hut;
pub mod newton;
pub mod newton_parallel;

pub trait Gravity: Send {
    /// Writes accelerations into the output parameter instead of returning a value to avoid
    /// heap allocation on every step by allowing buffer reuse in the main simulation loop.
    /// The accelerations buffer must be zeroed before each call to this function.
    fn calculate_accelerations(
        &mut self,
        masses: &[f64],
        rx: &[f64],
        ry: &[f64],
        rz: &[f64],
        ax: &mut [f64],
        ay: &mut [f64],
        az: &mut [f64],
    );
}
