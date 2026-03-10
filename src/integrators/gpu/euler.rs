use crate::gpu::{cuda_manager::CudaManager, device_bodies::DeviceBodies};
use crate::gravity::GpuGravity;
use crate::integrators::gpu::gpu_integrator::GpuIntegrator;
use cudarc::driver::PushKernelArg;

pub struct GpuEulerIntegrator {
    dt: f64,
}

impl GpuEulerIntegrator {
    pub fn new(dt: f64) -> Self {
        Self { dt }
    }
}

impl GpuIntegrator for GpuEulerIntegrator {
    fn step(
        &self,
        gpu: &CudaManager,
        bodies: &mut DeviceBodies,
        gravity: &dyn GpuGravity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let num_bodies = bodies.len() as u32;
        gravity.calculate_accelerations(gpu, bodies)?;

        let mut builder = gpu.stream.launch_builder(&gpu.cuda_fn_euler_step);
        builder
            .arg(&bodies.pos_x)
            .arg(&bodies.pos_y)
            .arg(&bodies.pos_z)
            .arg(&bodies.vel_x)
            .arg(&bodies.vel_y)
            .arg(&bodies.vel_z)
            .arg(&bodies.acc_x)
            .arg(&bodies.acc_y)
            .arg(&bodies.acc_z)
            .arg(&num_bodies)
            .arg(&self.dt);

        let config = cudarc::driver::LaunchConfig::for_num_elems(num_bodies);
        unsafe { builder.launch(config) }?;
        Ok(())
    }
}
