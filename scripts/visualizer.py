import sys
import pandas as pd
import numpy as np
import json
from dataclasses import dataclass, field
from PySide6.QtWidgets import QApplication, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QSlider, QLabel, QSizePolicy
from PySide6.QtCore import Qt, QTimer
from vispy import scene
from vispy.scene import visuals
from vispy.color import Color, ColorArray, get_colormap, get_color_names


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
        return cls(**data)
    
    def get_names(self, num_bodies) -> list[str]:
        return [
            self.names[i] if i < len(self.names) else f"Body {i + 1}"
            for i in range(num_bodies)
        ]
    
    def get_radii(self, num_bodies) -> np.ndarray:
        res = np.full(num_bodies, self.default_radius, dtype=np.float32)
        limit = min(num_bodies, len(self.radii))
        res[:limit] = self.radii[:limit]
        return res    
    
    def get_colors(self, num_bodies) -> np.ndarray:
        colormap = get_colormap('viridis')
        default_colors = colormap.map(np.linspace(0, 1, num_bodies))
        if not self.colors:
            return default_colors
        final_colors = np.empty((num_bodies, 4), dtype=np.float32)
        for i in range(num_bodies):
            if i < len(self.colors) and self.colors[i]:
                final_colors[i] = Color(self.colors[i]).rgba # type: ignore
            else:
                final_colors[i] = default_colors[i]
        return ColorArray(final_colors).rgba # type: ignore

        
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


class Visualizer(QWidget):
    def __init__(self, data: SimulationData, config: VisualizerConfig):
        super().__init__()
        self.data = data
        self.config = config
        self.initialize_data()
        self.initialize_state()
        self.initialize_ui()
        self.initialize_visuals()

    def initialize_data(self):
        self.num_bodies = len(self.data.ids)
        self.num_steps = len(self.data.times)
        self.body_paths = [self.data.positions[:, i, :] for i in range(self.num_bodies)]

    def initialize_state(self):
        self.step = 0
        self.playing = False
        self.timer = QTimer()
        self.timer.setInterval(round(1000 / self.config.step_rate))
        self.timer.timeout.connect(self.tick)

    def initialize_ui(self):
        self.setWindowTitle(self.config.window_title)
        self.setMinimumWidth(self.config.window_width)
        self.setMinimumHeight(self.config.window_height)
        layout = QVBoxLayout(self)

        header = QHBoxLayout()
        header.addWidget(QLabel(f"bodies: {self.num_bodies}, steps: {self.num_steps - 1}"))
        header.addStretch()
        self.time_label = QLabel("t = 0.00s")
        header.addWidget(self.time_label)
        layout.addLayout(header)

        # canvas setup
        self.canvas = scene.SceneCanvas(keys='interactive', show=False, bgcolor='black')
        self.canvas.native.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        layout.addWidget(self.canvas.native)

        # camera setup
        self.view = self.canvas.central_widget.add_view()
        self.view.camera = self.config.camera_mode

        # grid setup
        grid_size = 20  # Total span from -20 to 20
        grid_step = 1   # Distance between lines
        grid_points = []
        for i in range(-grid_size, grid_size + 1, grid_step):
            grid_points.extend([[-grid_size, i, 0], [grid_size, i, 0]]) # X-axis lines
            grid_points.extend([[i, -grid_size, 0], [i, grid_size, 0]]) # Y-axis lines
        self.grid = visuals.Line( # type: ignore
            pos=np.array(grid_points, dtype=np.float32),
            parent=self.view.scene,
            connect='segments',
            color=(0.5, 0.5, 0.5, 0.3),
            width=1
        )
        self.grid.transform = scene.transforms.STTransform(scale=(1, 1), translate=(0, 0, -0.01))
        # self.axis = visuals.XYZAxis(parent=self.view.scene) # type: ignore

        # controls setup
        controls = QHBoxLayout()
        self.play_btn = QPushButton("Play")
        self.play_btn.clicked.connect(self.toggle_play)
        controls.addWidget(self.play_btn)
        self.slider = QSlider(Qt.Orientation.Horizontal)
        self.slider.setRange(0, self.num_steps - 1)
        self.slider.valueChanged.connect(self.slider_changed)
        controls.addWidget(self.slider)
        layout.addLayout(controls)

    def initialize_visuals(self):
        self.body_names = self.config.get_names(self.num_bodies)
        self.body_radii = self.config.get_radii(self.num_bodies)
        self.body_colors = self.config.get_colors(self.num_bodies)
        self.trail_colors = self.body_colors.copy()
        self.trail_colors[:, 3] = 0.5

        # setup legend
        if self.config.enable_legend:
            x_offset = 20
            y_offset = 20
            spacing = 25
            indices = np.arange(self.num_bodies)
            y_positions = y_offset + (indices * spacing)
            dot_positions = np.zeros((self.num_bodies, 2))
            dot_positions[:, 0] = x_offset
            dot_positions[:, 1] = y_positions
            self.legend_dots = visuals.Markers(parent=self.canvas.scene) # type: ignore
            self.legend_dots.set_data(
                pos=dot_positions,
                face_color=self.body_colors,
                size=12,
                edge_width=0
            )
            label_positions = np.zeros((self.num_bodies, 2))
            label_positions[:, 0] = x_offset + 20
            label_positions[:, 1] = y_positions
            self.legend_labels = visuals.Text( # type: ignore
                text=self.body_names,
                parent=self.canvas.scene,
                color='white',
                font_size=10,
                anchor_x='left',
                anchor_y='center',
                pos=label_positions
            )

        # markers are the 3D points representing bodies
        self.markers = visuals.Markers( # type: ignore
            pos=self.data.positions[self.step],
            scaling=True,
            spherical=self.config.spherical,
            parent=self.view.scene,
            size=self.body_radii,
            edge_width=0,
            face_color=self.body_colors,
        ) 

        # trail lines are lines representing the entire path of the body
        self.trail_lines = []
        if self.config.enable_trails:
            for i in range(self.num_bodies):
                line = visuals.Line(parent=self.view.scene, color=self.trail_colors[i], width=2) # type: ignore
                self.trail_lines.append(line)

    def update(self):
        self.time_label.setText(f"t = {self.data.times[self.step]:.2f}s")
        self.markers.set_data(
            pos=self.data.positions[self.step],
            size=self.body_radii,
            edge_width=0,
            face_color=self.body_colors
        )
        
        if self.config.enable_trails:
            # only a segment of the last trail_window positions of the trail line is shown
            start_index = max(0, self.step - self.config.trail_window)
            for i, line in enumerate(self.trail_lines):
                segment = self.body_paths[i][start_index : self.step + 1]
                if len(segment) > 1:
                    line.set_data(pos=segment)
    
    def tick(self):
        self.step = (self.step + 1) % self.num_steps
        self.slider.blockSignals(True)
        self.slider.setValue(self.step)
        self.slider.blockSignals(False)
        self.update()

    def toggle_play(self):
        self.playing = not self.playing
        self.play_btn.setText("Pause" if self.playing else "Play")
        if self.playing: self.timer.start()
        else: self.timer.stop()

    def slider_changed(self, value):
        self.step = value
        self.update()

# Loads and transforms CSV simulation data into a SimulationData object
def load_from_csv(path: str) -> SimulationData:
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

def load_from_bin(path: str) -> SimulationData:
    # todo
    return SimulationData(np.empty(0), np.empty(0), np.empty(0))

def exit_with_error(message):
    print(message)
    input("\nPress Enter to close...")
    sys.exit(1)

if __name__ == '__main__':
    DEFAULT_DATA_PATH = "data/output.csv"

    # read simulation data file
    data_path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_DATA_PATH
    data_file_type = data_path.split(".")[-1].lower()

    if data_file_type == "csv":
        try:
            data = load_from_csv(data_path)
        except Exception as e:
            exit_with_error(f"Error: failed to load CSV data at {data_path}:\n{e}")
    elif data_file_type == "nbody":
        exit_with_error("Binary input format (.nbody) not supported yet")
        # try:
        #     data = load_from_bin(data_path)
        # except Exception as e:
        #     exit_with_error(f"Error: failed to load binary data at {data_path}:\n{e}")
    else:
        exit_with_error(f"Error: Unsupported data file type: .{data_file_type}\n"
                        "Simulation data must be in .csv or .nbody format.")

    # read visualization configuration file
    config_path = sys.argv[2] if len(sys.argv) > 2 else None
    
    if config_path:
        config_file_type = config_path.split(".")[-1].lower()
        if config_file_type == "json":
            try:
                config = VisualizerConfig.from_json(config_path)
            except Exception as e:
                exit_with_error(f"Error: failed to load JSON config at {config_path}:\n{e}")
        else:
            exit_with_error(f"Error: Unsupported config file type: .{config_file_type}\n"
                            "Configuration must be in .json format.")
    else:
        config = VisualizerConfig()

    try:
        app = QApplication(sys.argv)
        visualizer = Visualizer(data, config)
        visualizer.show()
        sys.exit(app.exec())
    except Exception as e:
        import traceback
        traceback.print_exc()
        exit_with_error("\nError: visualization failed")
