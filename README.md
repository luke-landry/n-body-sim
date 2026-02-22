# n-body-sim
A simulator for how multiple objects in space (bodies) move and interact with each other through gravity.

![cover](images/cover.png)

## Overview
This project implements an N-body simulator that models the gravitational interactions between bodies in 3D using various numerical integration methods. The physics engine is written in Rust, and the configuration and visualization GUIs are implemented in Python using the Qt framework and VisPy.

## Quick Start
### Prerequisites
- Python installed

### Windows Setup
1. Download and extract the latest Windows release zip
2. Run `install.bat` to setup the Python virtual environment and install required packages (first time only)
3. Run `run.bat` to start the application

### Linux Setup
1. Download and extract the latest Linux release tarball
2. Run `install.sh` to setup the Python virtual environment and install required packages (first time only)
    - On Debian/Ubuntu, you may need to install `python3-venv` with `sudo apt install python3-venv` before running this install script
3. Run `run.sh` to start the application

### Run an Example Simulation
1. Start the application using the instructions above
2. Click "Load Scenario" and select an example initial conditions file (e.g. `data/examples/figure-eight/initial_conditions.csv`)
3. Click "Launch and View Simulation" to run the simulation and see the results.
4. In the visualizer window, press the "Play" button in the bottom left to start the simulation playback.
5. Use the WASD, QE, and FC keys to move and left-click mouse-drag to look around
6. You can close the visualizer window at any time and go back to the launcher to load a different configuration or run a new simulation. The launcher stays open while the visualizer is running, so you can launch multiple simulations and have multiple visualizer windows open at the same time.

See the [GUI Usage](#gui-usage) and [CLI Usage](#cli-usage) sections below for more details on how to use the application.

## GUI Usage

### Launcher
The launcher allows you to configure simulation parameters and body initial conditions:
- **Simulation Parameters**: Set the various simulation parameters:
  - Gravitational constant 
  - Time step
  - Number of steps
  - Softening factor
  - Theta (for Barnes-Hut)
  - Gravity calculation method (if $N$ is the number of bodies in the simulation, it is recommended to use `newton` for $N\lt 100$, `newton-parallel` for $100 \leq N\lt 500$, and `barnes_hut` for $N\geq 500$.)
  - Integration method (the `runge-kutta` integrator takes the longest to compute, `velocity-verlet` is approximately twice as fast as `runge-kutta`, and `euler` is approximately twice as fast as `velocity-verlet`, so 4x faster than `runge-kutta`, but is the least accurate of the three)
- **Visualization Configuration**: Configure visualization and graphics settings
  - Camera mode (fly or turntable)
  - Step rate (playback speed in steps/second)
  - Default radius of bodies in visualization
  - Trail window size (length in previous time steps of trails behind each body)
  - Enable trails showing the recent path of each body ***(recommend disabling when viewing simulations with 100+ bodies to avoid lag during playback)***
  - Enable legend
  - Enable spherical visual effect for bodies (when disabled, bodies are rendered as flat colored circles)
- **Body Table**: Add, remove, and edit body properties (name, color, radius, mass, position, velocity)
- **Load/Save Scenario**: Load existing launcher configurations or save your current one
- **Generate Scenario**: Generate a random N-body system based on the selected generator (e.g. Star System), number of bodies (n) and radius (r).
- **Launch and View Simulation**: Start the physics simulation with your configured parameters and automatically display the visualization when complete
- **Launch Simulation**: Run the physics simulation without displaying the visualization. Prompts you to select a directory to save the results
- **View Simulation**: Load and display the visualization of previously saved simulation results from a selected directory

**Note:** The launcher expects these specific file names in selected directories when loading or saving configurations and simulation data:
- `initial_conditions.csv` for body initial conditions
- `config.json` for launcher configuration
- `output.csv` or `output.nbody` for simulation output data

When using the "Launch and View Simulation" option, the launcher automatically saves the initial conditions (`initial_conditions.csv`), the configuration (`config.json`), and the output data (`output.csv` or `output.nbody`) files to the directory `data/run/run_<timestamp>`. Otherwise, when using the individual "Launch Simulation" or "View Simulation" options, you will be prompted to select a directory to save or load these files, respectively.

### Visualizer

#### **Camera Navigation**
The visualizer supports two camera modes:

**Fly Mode** (free-flying camera):
- **Look**: Left-click and drag to look around
- **Movement**: Use WASD to move around, Q and E to roll, and F and C to move up and down
- **Zoom**: Scroll wheel to zoom in/out

**Turntable Mode** (orbit around center):
- **Rotate**: Left-click and drag to orbit around the center
- **Translate**: Shift-left-click and drag to translate the camera
- **Zoom**: Scroll wheel to zoom in/out

#### **Playback Controls**
- **Play/Pause**: Click the "Play" button to start/stop the simulation playback
- **Timeline**: Click and drag on the slider or click at any point to jump to a specific time step in the simulation

## CLI Usage
The Rust physics engine can also be run independently without installing Python or using the GUI tools.

### Command Line Options

**General Usage**
- `-i, --initial-conditions-path`: Path to a CSV file containing the initial conditions for each body in the simulation. Each row should represent a body with its mass, initial position, and initial velocity. Default: `initial_conditions.csv`
- `-o, --output-data-path`: Path to a CSV file where the simulation output data will be saved. Default: `output.csv`
- `-g, --g-constant`: The gravitational constant to use in the gravitational force calculations. This is a scaling factor that affects the strength of the gravitational interactions between bodies. Default: `1.0`
- `-t, --time-step`: The time step in seconds for the simulation. This determines how frequently the positions and velocities of the bodies are updated. A smaller time step can lead to more accurate results but will increase the computation time. Default: `0.01`
- `-n, --num-steps`: The total number of time steps to simulate. This determines the overall duration of the simulation. For example, with a time step of 0.01 seconds and 10000 steps, the simulation will cover a total of 100 seconds of simulated time. Default: `10000`
- `--softening-factor`: The softening factor is used to prevent numerical instability when two bodies come very close to each other. It is added to the distance between bodies in the force calculation to ensure that the force does not become infinite. A larger softening factor increases numerical stability but reduces physical accuracy at short distances, while a smaller softening factor provides higher physical accuracy but increases the risk of instability during close encounters. Default: `0.005`
- `--theta`: The theta value is used in the Barnes-Hut gravity calculation method to determine when to approximate a group of distant bodies as a single combined mass. A smaller theta value results in a more accurate simulation but increases computation time, while a larger theta value reduces accuracy but improves performance. Default: `0.5`
- `--gravity`: The method to use for calculating gravitational forces between bodies. The options are `newton`, `newton-parallel`, and `barnes-hut`. Default: `newton`
- `--integrator`: The numerical integration method to use for updating the positions and velocities of the bodies at each time step. The options are `euler`, `velocity-verlet`, and `runge-kutta`. Default: `euler`
- `-h --help`: Displays the help message with all available command line options, including those not listed here (e.g., benchmark options).

### Examples

**Windows:**
```
.\n-body-sim.exe -i data/examples/figure-eight.csv -o data/output.csv --time-step 0.01 --num-steps 10000 --integrator velocity-verlet
```

**Linux:**
```
./n-body-sim -i data/examples/figure-eight.csv -o data/output.csv --time-step 0.01 --num-steps 10000 --integrator velocity-verlet
```

## Data Formats

### Initial Conditions
Initial conditions are provided as a CSV file with each row corresponding to a body:
| Column  | Type  | Description        |
| ------- | ----- | ------------------ |
| `mass`  | float | Body mass          |
| `pos_x` | float | Initial x-position |
| `pos_y` | float | Initial y-position |
| `pos_z` | float | Initial z-position |
| `vel_x` | float | Initial x-velocity |
| `vel_y` | float | Initial y-velocity |
| `vel_z` | float | Initial z-velocity |

**Example**
```csv
mass,pos_x,pos_y,pos_z,vel_x,vel_y,vel_z
1,0,0,0,0,0,0
3.003e-6,1,0,0,0,1,0
3.694e-8,1.00257,0,0,0,1.0342,0
```

### Output Data

#### **CSV**

Output can be saved in a CSV file with time series data for all bodies:
| Column | Type  | Description           |
| ------ | ----- | ----------------------|
| `time` | float | Timestamp             |
| `id`   | int   | Body identifier       |
| `x`    | float | Current x-position    |
| `y`    | float | Current y-position    |
| `z`    | float | Current z-position    |

The body ID matches the order of the bodies in the initial conditions, so the first body is ID 0, the second is ID 1, and so on.

**Example**
```csv
time,id,x,y,z
0.0,0,-1.0,0.0,0.0
0.0,1,1.0,0.2,0.0
0.0,2,0.0,1.0,-0.2
0.01,0,-0.9999410506357825,0.005036782899891881,0.0019931360081177473
0.01,1,0.9969294480041954,0.1950342751323182,-9.184518199699633e-6
0.01,2,0.002011602631587234,0.9999289419677899,-0.19498395148991807
...
```

#### **Binary (*planned*)**

For better performance and smaller file sizes, especially for large simulations, output can be saved to a (little-endian) binary format:

| **Field** | **Type** | **Size (Bytes)** | **Description**                             |
| --------- | -------- | ---------------- | --------------------------------------------|
| `time`    | `f64`    | 8                | Timestamp                                   |
| `id`      | `u64`    | 8                | Body identifier                             |
| `x`       | `f64`    | 8                | The x-coordinate of the body's position.    |
| `y`       | `f64`    | 8                | The y-coordinate of the body's position.    |
| `z`       | `f64`    | 8                | The z-coordinate of the body's position.    |

The file extension for binary output data files is `.nbody` and every file begins with an 8-byte magic number: `0x4E424F4459303031` (ASCII `NBODY001`)

**Structure**
```
[0x4E424F4459303031][time][id][x][y][z][time][id][x][y][z]...
```

## Background

### Physics
The **N-body problem** involves predicting the individual motions of a group of objects interacting through gravitational force.
- **2-Body problem**: Systems with two objects (e.g., a planet and a moon) have a "closed-form" solution. A single mathematical formula can calculate their exact positions at any point in the future.
- **3-Body problem**: When a third object is added, the system becomes complex. Because the gravitational force on each object depends on the positions of all other objects, their motions are described by coupled differential equations. There is no general closed-form formula to solve these equations exactly. Instead, the system must be solved numerically by calculating the state of the system in small, successive time increments.

For 3+ bodies, the system generally becomes chaotic, which means it is highly sensitive to initial conditions. Two systems starting with a difference even as small as one millimeter in position will eventually diverge into completely different configurations.


### Numerical Methods & Algorithms

#### **Integrators**
Integrators are algorithms that update the position and velocity of each body at every time step.
- **Semi-implicit Euler**: A first-order symplectic integrator modified from the non-symplectic standard Euler method. It is simple and efficient but not the most accurate.
- **Velocity Verlet**: A second-order symplectic integrator that provides improved accuracy by evaluating accelerations at the beginning and end of each time step and using both to update positions and velocities.
- **Runge-Kutta**: The fourth-order Runge-Kutta method (RK4) is a non-symplectic integrator that achieves high accuracy by evaluating derivatives at four distinct points within each time step and combining them in a weighted average to update the state.


In non-symplectic integrators, such as the standard Euler or Runge-Kutta methods, numerical rounding errors accumulate, causing the system to gain or lose energy over time (e.g., planets spiraling into the sun). Symplectic integrators keep these energy errors bounded, ensuring that orbits remain stable over long simulation periods. Symplectic integrators are generally more accurate for long-term simulations while non-symplectic higher-order integrators may be preferred for short-term accuracy.

#### **Gravity**
These algorithms calculate the gravitational forces exerted on each body.
- **Newtonian**: Calculates the force between every pair of bodies directly. This is perfectly accurate but slow for large systems, with a time complexity of $O(n^2)$.
- **Barnes-Hut**: An algorithm used for large-scale simulation (e.g. galaxies). It organizes bodies into an octree, treating distant groups of objects as a single combined mass based on a given approximation threshold $\theta$ (theta). This introduces a small approximation error but significantly improves performance to $O(n\ log\ n)$.

#### **Softening Factor**
Gravitational force is calculated using Newton's Law of Universal Gravitation:

$$
F=G\frac{m_1m_2}{r^2}
$$

To prevent numerical singularities when two bodies pass very close to each other, this simulator uses a softening factor ($\epsilon$). When the distance ($r$) between bodies approaches zero, the ($1/r^2$) term approaches infinity, so this factor is added to the distance in the gravity force calculation to ensure it remains finite:

$$
F=G\frac{m_1m_2}{r^2+\epsilon^2}
$$

A larger $\epsilon$ increases numerical stability by smoothing out interactions, but it makes the simulation less physically accurate at short ranges. A smaller $\epsilon$ provides higher physical accuracy but increases the risk of numerical instability during close encounters.

## Build

### Container Setup
#### **Using Dev Containers**
1. Launch the project in a dev container from your code editor using the configuration in `.devcontainer/devcontainer.json`

#### **Manually with Docker installed**
1. Build the development image with `docker build -t n-body-sim .`
2. Run the development container with 
    - Linux (bash): `docker run -dit -v $(pwd):/home/dev/n-body-sim --name n-body-sim n-body-sim`
    - Windows (PS): `docker run -dit -v ${PWD}:/home/dev/n-body-sim --name n-body-sim n-body-sim`
3. Enter the container with `docker exec -it n-body-sim bash`


### Build Physics Engine Executable
  - Linux: `cargo build`
  - Windows: `cargo build --target x86_64-pc-windows-gnu`

## Benchmark
To run benchmarks for all combinations of gravity and integrator methods at various body counts, pass the `--benchmark` flag to the executable. Or simply run `cargo run --release -- --benchmark` from the project root. The benchmark results will be saved to `benchmark_results.csv` unless otherwise specified with the `--benchmark-output-path` flag. See all benchmark options using the `--help` flag.
