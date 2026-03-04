pub mod device_bodies;
pub mod gravity;
pub mod integrators;
pub mod simulation;

use std::error::Error;
use std::io::Error as IoError;
use std::panic; // for catching panics in GPU initialization
use std::sync::Arc;

use cudarc::driver::{CudaContext, CudaFunction, CudaStream, LaunchConfig};

// PTX instructions compiled by nvcc at build time are embedded
// here to be loaded as a module into the GPU at runtime
const PTX_DATA: &str = include_str!(env!("PTX_OUT"));

pub struct CudaManager {
    pub stream: Arc<CudaStream>,

    pub cuda_fn_gpu_init_check: CudaFunction,
    pub cuda_fn_newton_compute_accelerations: CudaFunction,
    pub cuda_fn_euler_step: CudaFunction,
}

impl CudaManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // save default panic hook and set a custom one to catch panics during GPU initialization
        let previous_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));

        let init_result = panic::catch_unwind(|| -> Result<Self, Box<dyn Error>> {
            let ctx = CudaContext::new(0)?;
            let stream = ctx.default_stream();
            let module = ctx.load_module(PTX_DATA.into())?;

            let cuda_fn_gpu_init_check = module.load_function("gpu_init_check")?;
            let cuda_fn_newton_compute_accelerations =
                module.load_function("newton_compute_accelerations")?;
            let cuda_fn_euler_step = module.load_function("euler_step")?;

            Ok(CudaManager {
                stream,
                cuda_fn_gpu_init_check,
                cuda_fn_newton_compute_accelerations,
                cuda_fn_euler_step,
            })
        });

        // restore the original panic hook after GPU initialization
        panic::set_hook(previous_hook);

        match init_result {
            Ok(result) => result,
            Err(_) => Err(IoError::other(
                "GPU initialization failed. Ensure an NVIDIA GPU/driver is installed or run without --gpu.",
            )
            .into()),
        }
    }

    pub fn gpu_init_check(&self) -> Result<(), Box<dyn Error>> {
        let mut builder = self.stream.launch_builder(&self.cuda_fn_gpu_init_check);
        unsafe { builder.launch(LaunchConfig::for_num_elems(1)) }?;
        self.stream.synchronize()?;
        Ok(())
    }
}
