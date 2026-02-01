import json
import pandas as pd
import numpy as np
from dataclasses import dataclass, field


@dataclass
class VisualizerConfig:
    window_title: str = "N-body 3D Visualizer"
    window_width: int = 1000
    window_height: int = 700
    step_rate: int = 100
    enable_trails: bool = True
    trail_window: int = 150
    camera_mode: str = 'fly'  # 'fly' or 'turntable'
    spherical: bool = True
    default_radius: float = 0.1
    enable_legend: bool = True
    names: list[str] = field(default_factory=list)
    radii: list[float] = field(default_factory=list)
    colors: list[list[float]] = field(default_factory=list)

    @classmethod
    def from_json(cls, path: str):
        with open(path, 'r') as f:
            data = json.load(f)
        if isinstance(data, dict) and "visualizerConfig" in data:
            data = data.get("visualizerConfig", {})
        return cls(**data)
    
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
def load_sim_data_from_csv(path: str) -> SimulationData:
    df = pd.read_csv(path)

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

    return SimulationData(positions, times, ids)

def load_sim_data_from_bin(path: str) -> SimulationData:
    # todo
    return SimulationData(np.empty(0), np.empty(0), np.empty(0))