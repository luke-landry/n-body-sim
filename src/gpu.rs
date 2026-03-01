use std::error::Error;
use std::sync::Arc;
use std::sync::LazyLock;

use cudarc::driver::{CudaContext, CudaFunction, CudaStream, LaunchConfig};

const PTX_DATA: &str = include_str!(env!("PTX_OUT"));

pub static GPU: LazyLock<CudaManager> =
    LazyLock::new(|| CudaManager::new().expect("Failed to initialize CUDA"));

pub struct CudaManager {
    stream: Arc<CudaStream>,

    cuda_fn_hello_gpu: CudaFunction,
}

impl CudaManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        println!("PTX content:\n{}", PTX_DATA);
        let ctx = CudaContext::new(0)?;
        let stream = ctx.default_stream();
        let module = ctx.load_module(PTX_DATA.into())?;
        let cuda_fn_hello_gpu = module.load_function("hello_gpu")?;
        Ok(CudaManager {
            stream,
            cuda_fn_hello_gpu,
        })
    }

    pub fn hello_gpu(&self) -> Result<(), Box<dyn Error>> {
        let mut builder = self.stream.launch_builder(&self.cuda_fn_hello_gpu);
        unsafe { builder.launch(LaunchConfig::for_num_elems(1)) }?;
        self.stream.synchronize()?;
        Ok(())
    }
}
