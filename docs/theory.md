# Theory

## The N-Body Problem

The **N-body problem** involves predicting the individual motions of a group of objects interacting through gravitational force.
- **2-Body problem**: Systems with two objects (e.g., a planet and a moon) have a "closed-form" solution. A single mathematical formula can calculate their exact positions at any point in the future.
- **3-Body problem**: When a third object is added, the system becomes complex. Because the gravitational force on each object depends on the positions of all other objects, their motions are described by coupled differential equations. There is no general closed-form formula to solve these equations exactly. Instead, the system must be solved numerically by calculating the state of the system in small, successive time increments.

For 3+ bodies, the system generally becomes chaotic, which means it is highly sensitive to initial conditions. Two systems starting with a difference even as small as one millimeter in position could eventually diverge into completely different configurations.

## Integrators

Integrators are algorithms that update the position and velocity of each body at every time step.
- **Semi-implicit Euler**: A first-order symplectic integrator modified from the non-symplectic standard Euler method. It is simple and efficient but not the most accurate.
- **Velocity Verlet**: A second-order symplectic integrator that provides improved accuracy by evaluating accelerations at the beginning and end of each time step and using both to update positions and velocities.
- **Runge-Kutta**: The fourth-order Runge-Kutta method (RK4) is a non-symplectic integrator that achieves high accuracy by evaluating derivatives at four distinct points within each time step and combining them in a weighted average to update the state.

In non-symplectic integrators, such as the standard Euler or Runge-Kutta methods, numerical rounding errors accumulate, causing the system to gain or lose energy over time (e.g., planets spiraling into the sun). Symplectic integrators keep these energy errors bounded, ensuring that orbits remain stable over long simulation periods. Symplectic integrators are generally more accurate for long-term simulations while non-symplectic higher-order integrators may be preferred for short-term accuracy.

## Gravity Methods

These algorithms calculate the gravitational forces exerted on each body.
- **Newton**: Calculates the force between every pair of bodies directly. This is perfectly accurate but slow for large systems, with a time complexity of $O(n^2)$.
- **Newton Parallel**: A multi-threaded version of the Newton method that calculates the forces on all bodies in parallel, improving performance for large systems compared to the single-threaded Newton method, but still has a time complexity of $O(n^2)$.
- **Barnes-Hut**: An algorithm used for large-scale simulation (e.g. galaxies). It organizes bodies into an octree, treating distant groups of objects as a single combined mass based on a given approximation threshold $\theta$ (theta). This introduces a small approximation error but significantly improves performance to $O(n\ log\ n)$.

While the **Newton** method is the slowest method in general, it has no threading overhead and so ends up being the fastest for small numbers of bodies. The **Newton Parallel** method becomes significantly faster than the single-threaded Newton method for larger numbers of bodies due to its use of multiple threads, but still has a quadratic time complexity. The **Barnes-Hut** method has a better time complexity than the Newton methods, and is also implemented as multi-threaded. However it has much more overhead from having to build and traverse an octree along with the threading overhead, so it is only most efficient in very large systems.

The **Barnes-Hut** approximation criterion is $\frac{s}{d} < \theta$, where $s$ is the size (width) of the cubic octree cell containing the group of bodies being approximated, $d$ is the distance from the center of mass of that cell to the body for which the force is being calculated, and $\theta$ is the approximation threshold. If this criterion is satisfied, it means the group of bodies in that cell is sufficiently far away and can be approximated as a single combined mass located at the center of mass of the cell. If not, the algorithm recursively checks the child cells of that octree cell until it finds cells that satisfy the approximation criterion or reaches leaf cells containing individual bodies. The smaller and further away a cell is, the more likely it is to satisfy the approximation criterion and be treated as a single combined mass, while closer and larger cells are more likely to fail the approximation criterion and are not approximated. By approximating distant groups of bodies as single masses, the Barnes-Hut algorithm reduces the number of force calculations needed, improving performance while introducing a small approximation error that can be controlled by adjusting the $\theta$ parameter. More information: [Barnes-Hut Simulation - Wikipedia](https://en.wikipedia.org/wiki/Barnes%E2%80%93Hut_simulation)

See [barnes-hut-octree.md](barnes-hut-octree.md) for a detailed breakdown of the octree data structure and implementation.

## Softening Factor

Gravitational force is calculated using Newton's Law of Universal Gravitation:

$$
F=G\frac{m_1m_2}{r^2}
$$

To prevent numerical singularities when two bodies pass very close to each other, this simulator uses a softening factor ($\epsilon$). When the distance ($r$) between bodies approaches zero, the ($1/r^2$) term approaches infinity, so this factor is added to the distance in the gravity force calculation to ensure it remains finite:

$$
F=G\frac{m_1m_2}{r^2+\epsilon^2}
$$

A larger $\epsilon$ increases numerical stability by smoothing out interactions, but it makes the simulation less physically accurate at short ranges. A smaller $\epsilon$ provides higher physical accuracy but increases the risk of numerical instability during close encounters.
