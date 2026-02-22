import random as rd

import numpy as np
from schema import BodyConfig


def calculate_orbital_velocity(G: float, M: float, r: float) -> float:
    return np.sqrt(G * M / r)


def generate_single_star_system(
    n: int,
    radius: float = 15.0,
    star_mass=1.0,
    star_radius=1.0,
    min_radius=2.0,
    max_inclination_deg=7.0,
    G=1.0,
) -> list[BodyConfig]:

    PLANET_MASS_MIN = 1e-5
    PLANET_MASS_MAX = 1e-2
    PLANET_RADIUS_MIN = 0.01
    PLANET_RADIUS_MAX = 0.2

    bodies: list[BodyConfig] = []
    inc = np.deg2rad(max_inclination_deg)

    # Central star
    bodies.append(
        BodyConfig(
            name="Star",
            color="#ffffaa",
            radius=star_radius,
            mass=star_mass,
            pos_x=0.0,
            pos_y=0.0,
            pos_z=0.0,
            vel_x=0.0,
            vel_y=0.0,
            vel_z=0.0,
        )
    )

    for i in range(1, n):
        # steps to generate planet with random orbit and
        # inclination about the main orbital (x-y) plane:
        #   1. Generate random position on the orbital plane and
        #      get base cartesian position on the flat disk
        #   2. Calculate orbital velocity and base cartesian components
        #   3. Rotate base position and velocity about the x axis (incline)
        #      then rotate randomly about the z axis (omega)

        # converting polar coordinates on the orbital plane to cartesian
        r = rd.uniform(min_radius, radius)
        theta = rd.uniform(0.0, 2.0 * np.pi)
        base_pos = np.array([r * np.cos(theta), r * np.sin(theta), 0.0])

        v = calculate_orbital_velocity(G, star_mass, r)
        base_vel = np.array([-v * np.sin(theta), v * np.cos(theta), 0.0])

        cos_inc = np.cos(inc)
        sin_inc = np.sin(inc)

        # omega is the longitude of the ascending node
        omega = rd.uniform(0.0, 2.0 * np.pi)
        cos_omega = np.cos(omega)
        sin_omega = np.sin(omega)

        # 3D rotation matrices
        Rx = np.array(
            [[1.0, 0.0, 0.0], [0.0, cos_inc, -sin_inc], [0.0, sin_inc, cos_inc]]
        )
        Rz = np.array(
            [[cos_omega, -sin_omega, 0.0], [sin_omega, cos_omega, 0.0], [0.0, 0.0, 1.0]]
        )

        # apply matrix multiplication to position and velocity vectors
        R = Rz @ Rx
        pos = R @ base_pos
        vel = R @ base_vel

        bodies.append(
            BodyConfig(
                name=f"Planet {i}",
                color=f"#{rd.randint(0, 0xFFFFFF):06x}",
                radius=rd.uniform(PLANET_RADIUS_MIN, PLANET_RADIUS_MAX),
                mass=rd.uniform(PLANET_MASS_MIN, PLANET_MASS_MAX),
                pos_x=pos[0],
                pos_y=pos[1],
                pos_z=pos[2],
                vel_x=vel[0],
                vel_y=vel[1],
                vel_z=vel[2],
            )
        )

    return bodies


def generate_disc_galaxy(
    n: int,
    radius: float = 20.0,
    body_mass: float = 1e-3,
    body_radius: float = 0.05,
    G: float = 1.0,
    density_power: float = 1.5,
    softening: float = 0.5,
) -> list[BodyConfig]:
    bodies: list[BodyConfig] = []

    # Generate positions
    radii = []
    positions = []
    for _ in range(n):
        r = radius * (rd.random() ** density_power)
        theta = rd.uniform(0, 2 * np.pi)
        z = rd.gauss(0, radius * 0.01)  # very thin initial disc
        radii.append(r)
        positions.append((r, theta, z))

    sorted_indices = np.argsort(radii)

    for rank, idx in enumerate(sorted_indices):
        r, theta, z = positions[idx]
        m_enclosed = (rank + 1) * body_mass
        v_mag = np.sqrt(G * m_enclosed * r / (r**2 + softening**2))

        # Add random velocity dispersion to prevent clumping
        dispersion = v_mag * 0.1
        vx = (-v_mag * np.sin(theta)) + rd.gauss(0, dispersion)
        vy = (v_mag * np.cos(theta)) + rd.gauss(0, dispersion)
        vz = rd.gauss(0, dispersion * 0.5)  # small vertical jitter

        x = r * np.cos(theta)
        y = r * np.sin(theta)

        bodies.append(
            BodyConfig(
                name=f"Star {idx}",
                color="#ffffff",
                radius=body_radius,
                mass=body_mass,
                pos_x=x,
                pos_y=y,
                pos_z=z,
                vel_x=vx,
                vel_y=vy,
                vel_z=vz,
            )
        )

    return bodies


generators = {
    "Star System": generate_single_star_system,
    "Disc Galaxy": generate_disc_galaxy,
}
