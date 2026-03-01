use core::panic;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cuda_src = root.join("cuda/acceleration.cu");
    if !cuda_src.exists() {
        panic!("CUDA source file not found at {}", cuda_src.display());
    }

    println!("cargo:rerun-if-changed={}", cuda_src.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ptx_out = out_dir.join("acceleration.ptx");

    compile_cuda(cuda_src, ptx_out);
}

/// Compiles CUDA source code into ptx files using nvcc.
/// The ptx is included in the binary at compile time  
/// and loaded at runtime using the cudarc crate
fn compile_cuda(cuda_src: PathBuf, ptx_out: PathBuf) {
    which::which("nvcc").expect(
        "nvcc not found in PATH. Please install CUDA toolkit and ensure nvcc is in your PATH.",
    );

    let status = Command::new("nvcc")
        .args(&[
            "-ptx",
            cuda_src.to_str().unwrap(),
            "-o",
            ptx_out.to_str().unwrap(),
        ])
        .status()
        .expect(
            "Failed to execute nvcc. Ensure CUDA toolkit is installed and nvcc is in your PATH.",
        );

    if !status.success() {
        panic!("nvcc failed to compile CUDA source. Check the error messages above for details.");
    }

    // set env variable with the ptx output path to the generated
    // ptx file so rustc can find it during compilation
    println!("cargo:rustc-env=PTX_OUT={}", ptx_out.display());
}
