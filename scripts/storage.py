import csv
import json
from pathlib import Path
from typing import Any, cast

import numpy as np
import pandas as pd
from schema import (
    BodyConfig,
    ScenarioConfig,
    SimulationData,
    SimulationParameters,
    VisualizerConfig,
)

IC_COLS = ["mass", "pos_x", "pos_y", "pos_z", "vel_x", "vel_y", "vel_z"]


def load_scenario(
    ic_csv_path: Path, config_json_path: Path | None = None
) -> tuple[SimulationParameters, VisualizerConfig, list[BodyConfig]]:

    # load initial conditions csv file
    if not ic_csv_path.exists():
        raise FileNotFoundError(f"Initial conditions CSV file not found: {ic_csv_path}")
    df = pd.read_csv(ic_csv_path)
    missing = set(IC_COLS) - set(df.columns)
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
                name=vis.names[i] if i < len(vis.names) else f"Body {i + 1}",
                color=vis.colors[i] if i < len(vis.colors) else "#ffffff",
                radius=vis.radii[i] if i < len(vis.radii) else vis.default_radius,
                **row,
            )
        )
    return sim, vis, bodies


def save_scenario(
    simulation_parameters: SimulationParameters,
    visualizer_config: VisualizerConfig,
    bodies: list[BodyConfig],
    ic_csv_path: Path,
    config_json_path: Path,
) -> None:
    if not bodies:
        raise ValueError("Cannot save scenario with empty body list")

    ic_csv_path.parent.mkdir(parents=True, exist_ok=True)
    config_json_path.parent.mkdir(parents=True, exist_ok=True)

    with ic_csv_path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=IC_COLS)
        writer.writeheader()
        for b in bodies:
            writer.writerow(b.model_dump(include=set(IC_COLS)))

    visualizer_config_out = visualizer_config.model_copy()
    visualizer_config_out.names = [b.name for b in bodies]
    visualizer_config_out.radii = [b.radius for b in bodies]
    visualizer_config_out.colors = [b.color for b in bodies]

    scenario = ScenarioConfig(
        simulation_parameters=simulation_parameters,
        visualizer_config=visualizer_config_out,
    )
    config_json_path.write_text(scenario.model_dump_json(indent=4), "utf-8", newline="")


def load_visualizer_config_from_json(path: Path) -> VisualizerConfig:
    if path.suffix != ".json":
        raise ValueError(f"Unsupported file format for config data: {path.suffix}")
    if not path.exists():
        raise FileNotFoundError(f"The config data file {path} does not exist")

    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)
        if isinstance(data, dict):
            data = data.get("visualizer_config", {})
        return VisualizerConfig(**data)


def load_simulation_data_from_csv(csv_path: Path) -> SimulationData:
    df = pd.read_csv(csv_path)

    # check for missing data
    if df.isnull().values.any():
        rows, cols = np.where(df.isnull())
        first_row = rows[0]
        first_col_name = df.columns[cols[0]]
        body_id = df.iloc[first_row].get("id", "Unknown").astype(int)
        time_val = df.iloc[first_row].get("time", "Unknown")
        raise ValueError(
            f"Missing value detected at CSV line {first_row + 2} "  # +2 for header offset
            f"(Column: '{first_col_name}', Time: {time_val}, ID: {body_id})."
        )

    df.set_index(["time", "id"], inplace=True)
    df.sort_index(inplace=True)

    times = df.index.get_level_values("time").unique().to_numpy().astype(np.float32)

    ids = df.index.get_level_values("id").unique().to_numpy().astype(np.uint32)

    # converts the time-series per-body xyz data into
    # a 3D array (tensor) of dimension (T, N, 3)
    positions = (
        df[["x", "y", "z"]]
        .to_numpy()
        .reshape(len(times), len(ids), 3)
        .astype(np.float32)
    )

    return SimulationData(positions, times, ids)


def load_simulation_data_from_bin(bin_path: Path) -> SimulationData:
    # TODO implement binary sim data file format
    raise RuntimeError("Binary simulation data format not supported yet")


def load_simulation_data_from_path(path: Path) -> SimulationData:
    if not path.exists():
        raise FileNotFoundError(f"The simulation data file {path} does not exist")
    if path.suffix == ".csv":
        data = load_simulation_data_from_csv(path)
    elif path.suffix == ".nbody":
        data = load_simulation_data_from_bin(path)
    else:
        raise ValueError(f"Unsupported file format for simulation data: {path.suffix}")
    return data
