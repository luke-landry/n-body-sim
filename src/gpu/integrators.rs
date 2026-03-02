pub mod euler;
// pub mod velocity_verlet; -- not implemented yet
// pub mod runge_kutta; -- not implemented yet

use crate::gpu::{CudaManager, device_bodies::DeviceBodies, gravity::GpuGravity};
use std::error::Error;

pub trait GpuIntegrator: Send {
    fn step(
        &self,
        gpu: &CudaManager,
        bodies: &mut DeviceBodies,
        gravity: &dyn GpuGravity, // Passed in, not owned
    ) -> Result<(), Box<dyn Error>>;
}
