from dataclasses import dataclass
from typing import Literal

import numpy as np
from pydantic import (
    BaseModel,
    ConfigDict,
    Field,
    NonNegativeFloat,
    NonNegativeInt,
    PositiveFloat,
    PositiveInt,
)


class SimulationParameters(BaseModel):
    g_constant: float = 1.0
    time_step: PositiveFloat = 0.025
    num_steps: PositiveInt = 10000
    softening_factor: float = 0.05
    theta: PositiveFloat = 0.5
    gravity: Literal["newton", "newton-parallel"] = "newton"
    integrator: Literal["euler", "velocity-verlet"] = "euler"
    progress: bool = False


class VisualizerConfig(BaseModel):
    window_title: str = "N-body 3D Visualizer"
    window_width: PositiveInt = 1000
    window_height: PositiveInt = 700
    step_rate: PositiveInt = 100
    enable_trails: bool = True
    trail_window: NonNegativeInt = 150
    camera_mode: Literal["fly", "turntable"] = "fly"
    spherical: bool = True
    default_radius: NonNegativeFloat = 0.1
    enable_legend: bool = True
    names: list[str] = Field(default_factory=list)
    radii: list[float] = Field(default_factory=list)
    colors: list[str] = Field(default_factory=list)


class ScenarioConfig(BaseModel):
    simulation_parameters: SimulationParameters
    visualizer_config: VisualizerConfig


class BodyConfig(BaseModel):
    model_config = ConfigDict(validate_assignment=True)
    name: str
    color: str
    radius: PositiveFloat
    mass: PositiveFloat
    pos_x: float
    pos_y: float
    pos_z: float
    vel_x: float
    vel_y: float
    vel_z: float

    @classmethod
    def default(cls, i: int) -> "BodyConfig":
        return cls(
            name=f"Body {i}",
            color="#ffffff",
            radius=0.1,
            mass=1.0,
            pos_x=0.0,
            pos_y=0.0,
            pos_z=0.0,
            vel_x=0.0,
            vel_y=0.0,
            vel_z=0.0,
        )


@dataclass
class SimulationData:
    positions: np.ndarray  # shape: (T, N, 3)
    times: np.ndarray  # shape: (T,)
    ids: np.ndarray  # shape: (N,)

    def __post_init__(self):
        if self.positions.ndim != 3:
            raise ValueError(f"positions must be 3D, got {self.positions.ndim}D")
        if self.times.ndim != 1:
            raise ValueError(f"times must be 1D, got {self.times.ndim}D")
        if self.ids.ndim != 1:
            raise ValueError(f"ids must be 1D, got {self.ids.ndim}D")

        t_pos, n_pos, d_pos = self.positions.shape
        t_time = self.times.shape[0]
        n_id = self.ids.shape[0]

        if d_pos != 3:
            raise ValueError(f"positions last dim must be 3, got {d_pos}")
        if t_pos != t_time:
            raise ValueError(f"T mismatch: positions has {t_pos}, times has {t_time}")
        if n_pos != n_id:
            raise ValueError(f"N mismatch: positions has {n_pos}, ids has {n_id}")

        if np.isnan(self.positions).any():
            raise ValueError(
                "Simulation positions contain NaN values. The input data is likely corrupt."
            )
