import random as rd

import numpy as np
from schema import BodyConfig


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

        # equation for orbital velocity is v = sqrt(GM/r)
        v = np.sqrt(G * star_mass / r)
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


generators = {"Star System": generate_single_star_system}
