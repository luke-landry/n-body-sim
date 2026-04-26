# Architecture

The project is organized into two main components: the Rust physics engine and the Python GUI tools. The physics engine is responsible for performing the N-body simulation, while the GUI tools provide an interface for configuring simulations and viewing results.

## Rust Physics Engine

Rust was chosen for the physics engine due to its performance, safety guarantees, and modern features. The engine is designed to be modular and extensible, allowing for easy addition of new integrators, gravity calculation methods, and other features in the future.

The source code of the Rust physics engine is in the `src` directory and contains the core logic for the N-body simulation, including the integrators, gravity calculation methods, and the core simulation loop. The integrators and gravity methods are implemented as traits, allowing for easy swapping and addition of new methods without modifying the core simulation logic. The core simulation loop iteratively updates the positions and velocities of the bodies based on the selected integrator and gravity method, and pushes the state of the system (body positions) at each step to a channel so as to not block the simulation for I/O operations, which are run in a separate thread. The simulation output data is streamed to a file in either CSV or binary format as it is generated, allowing for efficient handling of large simulations without consuming excessive memory storing huge generated datasets.

### Array of Structs vs Struct of Arrays

The initial conditions are read into a vector of structs `bodies: Vec<Body>`, each containing the mass, position, and velocity of a body. This format is known as "array of structs" (AoS) which is intuitive and easy to work with for reading and writing file data. However, it is not the most efficient data format for computation, as the pointer-chasing required to access the fields of each body struct can lead to slower performance. It is also not an ideal format for performing vectorized operations for one field on all bodies at once during the simulation calculations, which is able to be accelerated with SIMD instructions.

A more efficient format for computation is "struct of arrays" (SoA), where the body properties (mass, position, velocity) are stored in a struct `bodies: Bodies` containing equal-length arrays where each element corresponds to a body (e.g. for accessing the mass of the 5th body, it would be `bodies[5].mass` for AoS, and `bodies.masses[5]` in SoA). This allows for better cache locality because similar data is stored contiguously in memory, improving performance during the simulation calculations which often require operations on the properties of all bodies at once, such as updating the positions or velocities of all bodies in a step.

The simulation initial conditions are read from the CSV file into an AoS format, and then converted to an SoA format for the simulation calculations.

### GPU Acceleration

GPU acceleration was implemented by running the parallelizable parts of the simulation calculations on the GPU using CUDA. The body data is initially transferred to the GPU in SoA format for efficient computation, and the CPU passes pointers to the GPU memory where the body properties are stored each step, so that the GPU can directly read and write to those memory locations during the simulation calculations without needing to transfer large amounts of data back and forth between the CPU and GPU in between steps. To interact with GPU from Rust, the CUDA kernels are compiled to PTX intermediate representation at build time, and this PTX is included in the final executable. The `cudarc` crate is used to call the CUDA driver API to load the PTX at runtime and then call the CUDA kernel functions.

## Python GUI Tools

The GUI tools are implemented in Python using the Qt framework and VisPy. Python was chosen for the GUI due to its rapid development speed and the availability of convenient libraries for handling large datasets, creating GUIs, and making interactive visualizations.

The source code for the GUI tools is in the `gui` directory and contains the logic for the launcher and visualizer. The launcher allows users to configure simulation parameters, load initial conditions, generate random scenarios, and launch simulations. The visualizer provides an interactive 3D visualization of the simulation results, allowing users to play back the simulation and navigate the scene with different camera modes.

The launcher runs the Rust physics engine executable as a subprocess, passing the necessary configuration arguments and initial conditions file path. This way, the simulation runs independently of the launcher, allowing the launcher to remain responsive during a simulation.

It also runs the visualizer (which is a Python script) as a separate subprocess. The reason the visualizer is run as a separate subprocess instead of just being imported and called directly from the launcher is so that the visualizer doesn't block the launcher when it is loading the simulation output data, which can take a long time for large simulations. While this could also be solved by running the visualizer in a separate thread instead of a separate process, threads can only be safely terminated cooperatively, but if the visualizer thread is blocked on file I/O it cannot cooperatively check for termination signals from the launcher until it finishes loading, so the launcher would be unable to force-quit the visualizer if the user wanted to cancel loading early. By running the visualizer as a separate process, the launcher can simply kill the visualizer process, which is safe to do at any time.
