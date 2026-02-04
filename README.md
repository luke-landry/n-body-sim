# n-body-sim
A simulator for how multiple objects in space (bodies) move and interact with each other through gravity.

![cover](images/cover.png)

## Overview
This project implements an N-body simulator that models the gravitational interactions between bodies in 3D using numerical integration methods. The physics engine is written in Rust, and the configuration and visualization GUIs are implemented in Python using the Qt framework and VisPy.

## Quick Start
### Prerequisites
- Python installed

### Windows
1. Download and extract the latest Windows release zip
2. Run `install.bat` to setup the  virtual environment and install required packages (first time only)
3. Run `run.bat` to start the application

### Linux/macOS
1. Download and extract the latest Linux release tarball
2. Run `install.sh` to setup the virtual environment and install required packages (first time only)
3. Run `run.sh` to launch the application

## GUI Usage

### Launcher
The launcher allows you to configure simulation parameters and body initial conditions:
- **Parameter Configuration**: Set the various simulation parameters
- **Visualization Configuration**: Configure graphics settings
- **Body Table**: Add, remove, and edit body properties (name, color, radius, mass, position, velocity)
- **Load/Save**: Load existing configurations or save your current setup
- **Generate Random Scenario**: Generate a random N-body system
- **Launch Simulation**: Start the physics simulation with your configured parameters

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
- `-i, --initial-conditions-path <INITIAL_CONDITIONS_PATH>`: Path to initial conditions file
- `-o, --output-data-path <OUTPUT_DATA_PATH>`: Path to output trajectory data file
- `-g, --g-constant <G_CONSTANT>`: The gravitational constant to use in gravitational force calculations
- `-t, --time-step <TIME_STEP>`: Simulation time step in seconds
- `-n, --num-steps <NUM_STEPS>`: Number of time steps to simulate
- `--softening-factor <SOFTENING_FACTOR>`: The softening factor to avoid numerical instability as distances approach zero
- `--theta <THETA>`: Theta value for Barnes-Hut gravity calculation method
- `--gravity <GRAVITY>`: Gravity calculation method: `newton` (more to be added)
- `--integrator <INTEGRATOR>`: Integration method: `euler` (more to be added)
- `-h, --help`: Print help

### Examples

**Windows:**
```
.\n-body-sim.exe -i data/examples/figure-eight.csv -o data/output.csv --time-step 0.01 --num-steps 10000 --integrator euler
```

**Linux/macOS:**
```
./n-body-sim -i data/examples/figure-eight.csv -o data/output.csv --time-step 0.01 --num-steps 10000 --integrator euler
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
- **Semi-implicit Euler**: A first-order symplectic integrator. It is efficient but less precise over long durations.
- **Velocity Verlet (*planned*)**: A second-order symplectic integrator that provides a higher degree of precision by calculating positions and velocities at multiple points within the time step

The integrators used in this project are symplectic. In non-symplectic integrators, such as the standard Euler or Runge-Kutta methods, numerical rounding errors accumulate, causing the system to gain or lose energy over time (e.g., planets spiraling into the sun). Symplectic integrators keep these energy errors bounded, ensuring that orbits remain stable over long simulation periods.

#### **Gravity**
These algorithms calculate the gravitational forces exerted on each body.
- **Newtonian**: Calculates the force between every pair of bodies directly. This is perfectly accurate but slow for large systems, with a time complexity of $O(n^2)$.
- **Barnes-Hut (*planned*)**: An algorithm used for large-scale simulation (e.g. galaxies). It organizes bodies into a tree structure, treating distant groups of objects as a single combined mass. This introduces a small approximation error but significantly improves performance to $O(n\ log\ n)$.

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

### Using the VS Code Dev Containers extension
1. In VS Code, install the "Dev Containers" extension and then open this project
2. Launch the development container by following by selecting the `Reopen in Container` option in the Dev Containers notification, or use the `>Dev Containers: Rebuild and Reopen in Container` command in the command palette.
3. Run `cargo build` in the VS Code terminal

### Manually with Docker installed
1. Build the development image with `docker build -t n-body-sim .`
2. Run the development container with 
    - Linux (bash): `docker run -dit -v $(pwd):/home/dev/n-body-sim --name n-body-sim n-body-sim`
    - Windows (PS): `docker run -dit -v ${PWD}:/home/dev/n-body-sim --name n-body-sim n-body-sim`
3. Enter the container with `docker exec -it n-body-sim bash`
4. In the container, build the binary for Linux or .exe for Windows by running
    - Linux: `cargo build`
    - Windows: `cargo build --target x86_64-pc-windows-gnu`
