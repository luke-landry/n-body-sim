from pathlib import Path

import numpy as np
import storage
from PySide6.QtCore import Qt, QTimer
from PySide6.QtWidgets import (
    QApplication,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QSizePolicy,
    QSlider,
    QVBoxLayout,
    QWidget,
)
from schema import SimulationData, VisualizerConfig
from vispy import scene
from vispy.color import Color, ColorArray, get_colormap
from vispy.scene import visuals


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
        header.addWidget(
            QLabel(f"bodies: {self.num_bodies}, steps: {self.num_steps - 1}")
        )
        header.addStretch()
        self.time_label = QLabel("t = 0.00s")
        header.addWidget(self.time_label)
        layout.addLayout(header)

        # canvas setup
        self.canvas = scene.SceneCanvas(keys="interactive", show=False, bgcolor="black")
        self.canvas.native.setSizePolicy(
            QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding
        )
        layout.addWidget(self.canvas.native)

        # camera setup
        self.view = self.canvas.central_widget.add_view()
        self.view.camera = self.config.camera_mode

        # grid setup
        grid_size = 20  # Total span from -20 to 20
        grid_step = 1  # Distance between lines
        grid_points = []
        for i in range(-grid_size, grid_size + 1, grid_step):
            grid_points.extend([[-grid_size, i, 0], [grid_size, i, 0]])  # X-axis lines
            grid_points.extend([[i, -grid_size, 0], [i, grid_size, 0]])  # Y-axis lines
        self.grid = visuals.Line(  # type: ignore
            pos=np.array(grid_points, dtype=np.float32),
            parent=self.view.scene,  # type: ignore
            connect="segments",
            color=(0.5, 0.5, 0.5, 0.3),
            width=1,
        )
        self.grid.transform = scene.transforms.STTransform(
            scale=(1, 1), translate=(0, 0, -0.01)
        )
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
        self.body_names = self.generate_names()
        self.body_radii = self.generate_radii()
        self.body_colors = self.generate_colors()
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
            self.legend_dots = visuals.Markers(parent=self.canvas.scene)  # type: ignore
            self.legend_dots.set_data(
                pos=dot_positions,
                face_color=self.body_colors,  # type: ignore
                size=12,
                edge_width=0,  # type: ignore
            )
            label_positions = np.zeros((self.num_bodies, 2))
            label_positions[:, 0] = x_offset + 20
            label_positions[:, 1] = y_positions
            self.legend_labels = visuals.Text(  # type: ignore
                text=self.body_names,
                parent=self.canvas.scene,  # type: ignore
                color="white",
                font_size=10,
                anchor_x="left",
                anchor_y="center",
                pos=label_positions,
            )

        # markers are the 3D points representing bodies
        self.markers = visuals.Markers(  # type: ignore
            pos=self.data.positions[self.step],
            scaling=True,  # type: ignore
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
                line = visuals.Line(
                    parent=self.view.scene,  # type: ignore
                    color=self.trail_colors[i],
                    width=2,  # type: ignore
                )  # type: ignore
                self.trail_lines.append(line)

    def update_plot(self):
        self.time_label.setText(f"t = {self.data.times[self.step]:.2f}s")
        self.markers.set_data(
            pos=self.data.positions[self.step],
            size=self.body_radii,  # type: ignore
            edge_width=0,
            face_color=self.body_colors,  # type: ignore
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
        self.update_plot()

    def toggle_play(self):
        self.playing = not self.playing
        self.play_btn.setText("Pause" if self.playing else "Play")
        if self.playing:
            self.timer.start()
        else:
            self.timer.stop()

    def slider_changed(self, value):
        self.step = value
        self.update_plot()

    def generate_names(self) -> list[str]:
        return [
            self.config.names[i] if i < len(self.config.names) else f"Body {i + 1}"
            for i in range(self.num_bodies)
        ]

    def generate_radii(self) -> np.ndarray:
        res = np.full(self.num_bodies, self.config.default_radius, dtype=np.float32)
        limit = min(self.num_bodies, len(self.config.radii))
        res[:limit] = self.config.radii[:limit]
        return res

    def generate_colors(self) -> np.ndarray:
        colormap = get_colormap("viridis")
        default_colors = colormap.map(np.linspace(0, 1, self.num_bodies))
        if not self.config.colors:
            return default_colors
        final_colors = np.empty((self.num_bodies, 4), dtype=np.float32)
        for i in range(self.num_bodies):
            if i < len(self.config.colors) and self.config.colors[i]:
                final_colors[i] = Color(self.config.colors[i]).rgba  # type: ignore
            else:
                final_colors[i] = default_colors[i]
        return ColorArray(final_colors).rgba  # type: ignore


# visualizer.py can be run standalone as a script
if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print(
            "Visualizer script needs at least one arg:\n"
            "\tpath/to/data (.csv/.nbody)"
            "\toptional: path/to/config (.json) "
        )

    data_path = Path(sys.argv[1])
    config_path = Path(sys.argv[2]) if len(sys.argv) > 2 else None

    try:
        app = QApplication(sys.argv)
        data = storage.load_simulation_data_from_path(data_path)
        config = (
            storage.load_visualizer_config_from_json(config_path)
            if config_path
            else VisualizerConfig()
        )
        visualizer = Visualizer(data, config)
        visualizer.show()
        sys.exit(app.exec())
    except Exception as e:
        print(f"\nError: visualization failed: {e}")
