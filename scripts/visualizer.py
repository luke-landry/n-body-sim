import sys, signal
import pandas as pd
import numpy as np
from PySide6.QtWidgets import QApplication, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QSlider, QLabel
from PySide6.QtCore import Qt, QTimer
# pyqtgraph needs to be imported after PySide6 to use it, otherwise it will try to use PyQt6
import pyqtgraph as pg


WINDOW_TITLE = "N-body Simulation Visualizer"

DEFAULT_FILENAME = "data/output.csv"

# Default number of steps to show per second
DEFAULT_STEP_RATE = 60

POINT_SIZE = 10

TRAIL_WINDOW = 150

TRAIL_ALPHA = 150

INITIAL_MARGIN = 5

TRAIL_Z = 1

BODY_Z = 10


class Visualizer(QWidget):
    def __init__(self, df: pd.DataFrame, path, step_rate):
        super().__init__()

        self.path = path

        self.step = 0
        self.playing = False

        self.timer = QTimer()
        self.timer.setInterval(round(1000 / step_rate))
        self.timer.timeout.connect(self.tick)

        self.initialize_data(df)
        self.initialize_ui()
        self.initialize_plot()
        self.update(0)

    def initialize_data(self, df: pd.DataFrame):
        self.times = df.index.unique(level='time').to_numpy(dtype=float)
        self.bodies = df.index.unique(level='id').to_numpy(dtype=int)
        num_steps = len(self.times)
        num_bodies = len(self.bodies)

        # Precompute 3D numpy array [times, bodies, (x,y)]
        self.data = np.empty((num_steps, num_bodies, 2), dtype=np.float64)
        for body in self.bodies :
            body_slice = df.xs(body, level='id')
            self.data[:, body, 0] = body_slice['x'].to_numpy()
            self.data[:, body, 1] = body_slice['y'].to_numpy()

        # Compute metadata
        initial_df = df.xs(self.times[0], level='time')
        self.ix_min, self.ix_max = initial_df["x"].min(), initial_df["x"].max()
        self.iy_min, self.iy_max = initial_df["y"].min(), initial_df["y"].max()
    
    def initialize_ui(self):
        self.setWindowTitle(WINDOW_TITLE)
        layout = QVBoxLayout(self)

        header = QHBoxLayout()
        layout.addLayout(header)

        self.file_label = QLabel(f"N-body simulation data loaded from {self.path}")
        header.addWidget(self.file_label)

        header.addStretch()

        self.time_label = QLabel("t = 0.00s")
        header.addWidget(self.time_label)

        margin_x = abs(self.ix_max - self.ix_min) * INITIAL_MARGIN
        margin_y = abs(self.iy_max - self.iy_min) * INITIAL_MARGIN

        self.plot = pg.PlotWidget()
        self.plot.setAspectLocked(True)
        self.plot.showGrid(x=True, y=True)
        self.plot.setXRange(self.ix_min, self.ix_max, padding=margin_x) # type: ignore
        self.plot.setYRange(self.iy_min, self.iy_max, padding=margin_y) # type: ignore
        layout.addWidget(self.plot)  # type: ignore

        controls = QHBoxLayout()
        layout.addLayout(controls)

        self.play_btn = QPushButton("Play")
        self.play_btn.clicked.connect(self.toggle_play)
        controls.addWidget(self.play_btn)

        self.slider = QSlider(Qt.Orientation.Horizontal)
        self.slider.setMinimum(0)
        self.slider.setMaximum(len(self.times) -1)
        self.slider.valueChanged.connect(self.slider_changed)
        controls.addWidget(self.slider)

    def initialize_plot(self):
        legend = self.plot.addLegend()
        legend.setBrush(pg.mkBrush(0, 0, 0, 175))

        self.trails: dict[int, pg.PlotDataItem] = {}
        self.points: dict[int, pg.ScatterPlotItem]  = {}
        for body in self.bodies:
            color = pg.intColor(int(body), hues=len(self.bodies))

            trail_color = pg.mkColor(color)
            trail_color.setAlpha(TRAIL_ALPHA)
            self.trails[body] = pg.PlotDataItem(pen=pg.mkPen(color=trail_color, width=1))
            self.trails[body].setZValue(TRAIL_Z)
            self.plot.addItem(self.trails[body])

            self.points[body] = pg.ScatterPlotItem(size=POINT_SIZE, brush=color, name=f"Body {body + 1}")
            self.points[body].setZValue(BODY_Z)
            self.plot.addItem(self.points[body])
            
    def update(self, step):
        self.time_label.setText(f"t = {self.times[step]:.2f}s")
        starting_step = max(0, step - TRAIL_WINDOW)
        for body in self.bodies:
            trail = self.data[starting_step : step + 1, body]
            self.trails[body].setData(trail[:, 0], trail[:, 1])
            self.points[body].setData([trail[-1, 0]], [trail[-1, 1]])

    def tick(self):
        self.update(self.step)
        self.slider.blockSignals(True)
        self.slider.setValue(self.step)
        self.slider.blockSignals(False)
        self.step = (self.step + 1) % len(self.times)
    
    def toggle_play(self):
        self.playing = not self.playing
        self.play_btn.setText("Pause" if self.playing else "Play")
        if self.playing:
            self.timer.start()
        else:
            self.timer.stop()

    def slider_changed(self, value):
        self.step = value
        self.update(self.step)

# Parse args
path_str = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_FILENAME
step_rate = sys.argv[2] if len(sys.argv) > 2 else DEFAULT_STEP_RATE

# read csv data into a dataframe
print(f"Reading simulation data from {path_str}")
df  = pd.DataFrame()
try:
    df = pd.read_csv(path_str)
except FileNotFoundError:
    print(f"Error: file '{path_str}' not found.")
    sys.exit(1)
df.set_index(['time', 'id'], inplace=True)
df.sort_index(inplace=True)

# parse step rate
try:
    step_rate = int(step_rate)
except ValueError:
    print(f"Error: invalid step rate {step_rate}")
    sys.exit(1)

# launch visualizer qt app
print("Launching visualization window")
app = QApplication()
visualizer = Visualizer(df, path_str, step_rate)
visualizer.show()

# this allows ctrl+c to work while the window is launched
signal.signal(signal.SIGINT, signal.SIG_DFL)

print("Visualization launched, press play in the bottom left of the window to start")
print("To return to this terminal, close the simulation window or press Ctrl+C here")

sys.exit(app.exec())
