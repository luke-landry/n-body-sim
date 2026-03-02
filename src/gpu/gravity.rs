// pub mod newton; -- not implemented: single-threaded Newton does not make sense on a GPU
pub mod newton_parallel;
// pub mod barnes_hut; -- not implemented yet

use crate::gpu;
use crate::gpu::CudaManager;
use std::error::Error;

pub trait GpuGravity: Send {
    fn calculate_accelerations(
        &self,
        gpu: &CudaManager,
        bodies: &gpu::device_bodies::DeviceBodies,
    ) -> Result<(), Box<dyn Error>>;
}
