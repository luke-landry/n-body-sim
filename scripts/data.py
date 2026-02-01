import json
import pandas as pd
import numpy as np
import csv
from pathlib import Path
from dataclasses import dataclass
from typing import Literal, Any, cast, Self
from pydantic import BaseModel, PositiveFloat, PositiveInt, NonNegativeInt, NonNegativeFloat, Field, ConfigDict

class SimulationParameters(BaseModel):
    g_constant: float = 1.0
    time_step: PositiveFloat = 0.01
    num_steps: PositiveInt = 10000
    softening_factor: float = 0.005

class VisualizerConfig(BaseModel):
    window_title: str = "N-body 3D Visualizer"
    window_width: PositiveInt = 1000
    window_height: PositiveInt = 700
    step_rate: PositiveInt = 100
    enable_trails: bool = True
    trail_window: NonNegativeInt = 150
    camera_mode: Literal['fly', 'turntable'] = 'fly'
    spherical: bool = True
    default_radius: NonNegativeFloat = 0.1
    enable_legend: bool = True
    names: list[str] = Field(default_factory=list)
    radii: list[float] = Field(default_factory=list)
    colors: list[str] = Field(default_factory=list)

    @classmethod
    def from_json(cls, path):
        with open(path, 'r') as f:
            data = json.load(f)
        if isinstance(data, dict):
            data = data.get("visualizer_config", {})
        return cls(**data)

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
    
    
def load_scenario(ic_csv_path: Path,
                  config_json_path: Path | None = None
                  ) -> tuple[
                      SimulationParameters,
                      VisualizerConfig,
                      list[BodyConfig]]:

    # load initial conditions csv file
    if not ic_csv_path.exists():
        raise FileNotFoundError(f"Initial conditions CSV file not found: {ic_csv_path}")
    df = pd.read_csv(ic_csv_path)
    expected_cols = {
        "mass", "pos_x", "pos_y", "pos_z",
        "vel_x", "vel_y", "vel_z",
    }
    missing = expected_cols - set(df.columns)
    if missing:
        raise ValueError(f"Missing CSV columns: {missing}")
    
    records: list[dict[str, Any]] = cast(
        list[dict[str, Any]],
        df.to_dict("records"),
    )
    
    # load scenario config json file
    if config_json_path is not None:
        if not config_json_path.exists():
            raise FileNotFoundError(f"Config JSON file not found: {config_json_path}")
        scenario = ScenarioConfig.model_validate_json(
            config_json_path.read_text("utf-8")
        )
        sim = scenario.simulation_parameters
        vis = scenario.visualizer_config
    else:
        sim = SimulationParameters()
        vis = VisualizerConfig()

    bodies: list[BodyConfig] = []
    for i, row in enumerate(records):
        bodies.append(
            BodyConfig(
                name=vis.names[i] if i < len(vis.names) else f"Body {i+1}",
                color=vis.colors[i] if i < len(vis.colors) else "#ffffff",
                radius=vis.radii[i] if i < len(vis.radii) else vis.default_radius,
                **row,
            )
        )
    return sim, vis, bodies
    
def save_scenario(simulation_parameters: SimulationParameters, visualizer_config: VisualizerConfig, bodies: list[BodyConfig],
                  ic_csv_path: Path, config_json_path: Path) -> None:
    if not bodies:
        raise ValueError("Cannot save scenario with empty body list")
    
    ic_csv_path.parent.mkdir(parents=True, exist_ok=True)
    config_json_path.parent.mkdir(parents=True, exist_ok=True)

    ic_csv_keys = [
        "mass", "pos_x", "pos_y", "pos_z",
        "vel_x", "vel_y", "vel_z",
    ]
    
    with ic_csv_path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=ic_csv_keys)
        writer.writeheader()
        for b in bodies:
            writer.writerow(b.model_dump(include=set(ic_csv_keys)))

    visualizer_config_out = visualizer_config.model_copy()
    visualizer_config_out.names = [b.name for b in bodies]
    visualizer_config_out.radii = [b.radius for b in bodies]
    visualizer_config_out.colors = [b.color for b in bodies]

    scenario = ScenarioConfig(
        simulation_parameters=simulation_parameters,
        visualizer_config=visualizer_config_out,
    )
    config_json_path.write_text(
        scenario.model_dump_json(indent=4),
        "utf-8",
    )
    
@dataclass
class SimulationData:
    positions: np.ndarray   # shape: (T, N, 3)
    times: np.ndarray       # shape: (T,)
    ids: np.ndarray         # shape: (N,)

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
            raise ValueError("Simulation positions contain NaN values. The input data is likely corrupt.")
            
    # Loads and transforms CSV simulation data into a SimulationData object
    @classmethod
    def from_csv(cls, csv_path: Path) -> Self:
        df = pd.read_csv(csv_path)

        # check for missing data
        if df.isnull().values.any():
            rows, cols = np.where(df.isnull())
            first_row = rows[0]
            first_col_name = df.columns[cols[0]]
            body_id = df.iloc[first_row].get('id', 'Unknown').astype(int)
            time_val = df.iloc[first_row].get('time', 'Unknown')
            raise ValueError(
                f"Missing value detected at CSV line {first_row + 2} " # +2 for header offset
                f"(Column: '{first_col_name}', Time: {time_val}, ID: {body_id})."
            )

        df.set_index(["time", "id"], inplace=True)
        df.sort_index(inplace=True)
        
        times = (df.index
                .get_level_values("time")
                .unique()
                .to_numpy()
                .astype(np.float32)
        )
        
        ids = (df.index
            .get_level_values("id")
            .unique()
            .to_numpy()
            .astype(np.uint32)
        )

        # converts the time-series per-body xyz data into
        # a 3D array (tensor) of dimension (T, N, 3)
        positions = (
            df[["x", "y", "z"]]
            .to_numpy()
            .reshape(len(times), len(ids), 3)
            .astype(np.float32)
        )

        return cls(positions, times, ids)

    @classmethod
    def from_bin(cls, bin_path: Path) -> Self:
        #TODO implement binary sim data file format
        raise RuntimeError("Binary simulation data not supported yet")
