import sys
from datetime import datetime
from pathlib import Path

import generators
import storage
from models import BodyTableModel
from PySide6.QtCore import Signal
from PySide6.QtWidgets import (
    QCheckBox,
    QComboBox,
    QDoubleSpinBox,
    QFileDialog,
    QFormLayout,
    QGroupBox,
    QHBoxLayout,
    QHeaderView,
    QPushButton,
    QSpinBox,
    QTableView,
    QVBoxLayout,
    QWidget,
)
from schema import BodyConfig, SimulationParameters, VisualizerConfig

INITIAL_CONDITIONS_FILENAME = "initial_conditions.csv"
CONFIG_FILENAME = "config.json"
OUTPUT_FILENAME = "output.csv"


# main menu for configuring, launching, and viewing a simulation
class Launcher(QWidget):
    run_sim = Signal(
        Path,  # directory path
        SimulationParameters,  # simulation parameters
        list,  # body configs
        VisualizerConfig,  # visualizer config
        bool,  # auto-run the visualizer afterwards
    )
    view_sim = Signal(Path)
    error = Signal(str)

    def __init__(self):
        super().__init__()

        BASE_PATH = Path(__file__).parents[1]
        self.RUN_DIR_PATH = BASE_PATH / Path("data/run")
        self.RUN_DIR_PATH.mkdir(parents=True, exist_ok=True)

        self.initialize_ui()
        print("Launcher started")

    def initialize_ui(self):
        self.setWindowTitle("N-Body Launcher")
        self.setMinimumSize(700, 500)
        self.main_layout = QVBoxLayout()
        self.setLayout(self.main_layout)

        self.params_group = QHBoxLayout()
        self.main_layout.addLayout(self.params_group)

        self.sim_group = QGroupBox("Simulation Parameters")
        self.params_group.addWidget(self.sim_group, 1)
        self.vis_group = QGroupBox("Visualization Config")
        self.params_group.addWidget(self.vis_group, 1)

        self.initialize_ui_simulation_parameters()
        self.initialize_ui_visualization_config()
        self.initialize_ui_body_table()
        self.initialize_ui_controls()

    def initialize_ui_simulation_parameters(self):
        sim_form_layout = QFormLayout()
        default_sim = SimulationParameters()

        # gravitational Constant (G)
        self.g_input = QDoubleSpinBox()
        self.g_input.setDecimals(15)  # High precision for G
        self.g_input.setRange(0, 1)
        self.g_input.setValue(default_sim.g_constant)
        sim_form_layout.addRow("G Constant:", self.g_input)

        # time Step (t)
        self.dt_input = QDoubleSpinBox()
        self.dt_input.setDecimals(4)
        self.dt_input.setRange(0.0001, 100000)
        self.dt_input.setValue(default_sim.time_step)
        sim_form_layout.addRow("Time Step (s):", self.dt_input)

        # number of Steps (n)
        self.steps_input = QSpinBox()
        self.steps_input.setRange(1, 10000000)
        self.steps_input.setSingleStep(1000)
        self.steps_input.setValue(default_sim.num_steps)
        sim_form_layout.addRow("Num Steps:", self.steps_input)

        # softening Factor
        self.softening_input = QDoubleSpinBox()
        self.softening_input.setDecimals(6)
        self.softening_input.setRange(0, 100)
        self.softening_input.setValue(default_sim.softening_factor)
        sim_form_layout.addRow("Softening Factor:", self.softening_input)

        # theta for Barnes-Hut
        self.theta_input = QDoubleSpinBox()
        self.theta_input.setDecimals(4)
        self.theta_input.setRange(0.0, 10.0)
        self.theta_input.setValue(default_sim.theta)
        sim_form_layout.addRow("Theta (Barnes-Hut):", self.theta_input)

        # gravity method
        self.gravity_input = QComboBox()
        self.gravity_input.addItems(["newton", "newton-parallel"])
        self.gravity_input.setCurrentText(default_sim.gravity)
        sim_form_layout.addRow("Gravity Method:", self.gravity_input)

        # integrator method
        self.integrator_input = QComboBox()
        self.integrator_input.addItems(["euler", "velocity-verlet", "runge-kutta"])
        self.integrator_input.setCurrentText(default_sim.integrator)
        sim_form_layout.addRow("Integrator Method:", self.integrator_input)

        self.sim_group.setLayout(sim_form_layout)

    def initialize_ui_visualization_config(self):
        vis_form_layout = QFormLayout()
        default_vis = VisualizerConfig()

        # camera mode
        self.camera_mode_input = QComboBox()
        self.camera_mode_input.addItems(["fly", "turntable"])
        self.camera_mode_input.setCurrentText(default_vis.camera_mode)
        vis_form_layout.addRow("Camera Mode:", self.camera_mode_input)

        # step rate
        self.step_rate_input = QSpinBox()
        self.step_rate_input.setRange(1, 10000)
        self.step_rate_input.setValue(default_vis.step_rate)
        vis_form_layout.addRow("Step Rate:", self.step_rate_input)

        # default radius
        self.default_radius_input = QDoubleSpinBox()
        self.default_radius_input.setDecimals(4)
        self.default_radius_input.setRange(0, 100)
        self.default_radius_input.setValue(default_vis.default_radius)
        vis_form_layout.addRow("Default Radius:", self.default_radius_input)

        # trail window
        self.trail_window_input = QSpinBox()
        self.trail_window_input.setRange(0, 10000)
        self.trail_window_input.setValue(default_vis.trail_window)
        vis_form_layout.addRow("Trail Window:", self.trail_window_input)

        # enable trails
        self.enable_trails_input = QCheckBox()
        self.enable_trails_input.setChecked(default_vis.enable_trails)
        vis_form_layout.addRow("Enable Trails:", self.enable_trails_input)

        # enable legend
        self.enable_legend_input = QCheckBox()
        self.enable_legend_input.setChecked(default_vis.enable_legend)
        vis_form_layout.addRow("Enable Legend:", self.enable_legend_input)

        # spherical
        self.spherical_input = QCheckBox()
        self.spherical_input.setChecked(default_vis.spherical)
        vis_form_layout.addRow("Spherical:", self.spherical_input)

        self.vis_group.setLayout(vis_form_layout)

    def initialize_ui_body_table(self):
        # table of body configurations
        self.body_table_model = BodyTableModel()
        self.body_table_view = QTableView()
        self.body_table_view.setModel(self.body_table_model)
        self.body_table_view.horizontalHeader().setSectionResizeMode(
            QHeaderView.Stretch  # type: ignore
        )  # type: ignore
        self.main_layout.addWidget(self.body_table_view)

    def initialize_ui_controls(self):
        # button controls layout
        button_layout = QHBoxLayout()
        self.add_btn = QPushButton("Add Body")
        self.add_btn.clicked.connect(self.body_table_model.add_body)
        button_layout.addWidget(self.add_btn)
        self.remove_btn = QPushButton("Delete Selected Body")
        self.remove_btn.clicked.connect(self.remove_selected_body)
        button_layout.addWidget(self.remove_btn)
        self.load_btn = QPushButton("Load Configuration")
        self.load_btn.clicked.connect(self.handle_load)
        button_layout.addWidget(self.load_btn)
        self.save_btn = QPushButton("Save Configuration")
        self.save_btn.clicked.connect(self.handle_save)
        button_layout.addWidget(self.save_btn)
        self.main_layout.addLayout(button_layout)

        # generator controls layout
        generator_layout = QHBoxLayout()
        self.generator_combo = QComboBox()
        self.generator_combo.addItems(list(generators.generators.keys()))
        generator_layout.addWidget(self.generator_combo)
        self.generator_n_input = QSpinBox()
        self.generator_n_input.setPrefix("n: ")
        self.generator_n_input.setRange(2, 1000)
        self.generator_n_input.setValue(10)
        generator_layout.addWidget(self.generator_n_input)
        self.generator_r_input = QDoubleSpinBox()
        self.generator_r_input.setPrefix("r: ")
        self.generator_r_input.setDecimals(2)
        self.generator_r_input.setRange(1.0, 100.0)
        self.generator_r_input.setValue(15.0)
        generator_layout.addWidget(self.generator_r_input)
        self.generate_btn = QPushButton("Generate Scenario")
        self.generate_btn.clicked.connect(self.handle_generate)
        generator_layout.addWidget(self.generate_btn)
        self.main_layout.addLayout(generator_layout)

        # launch button
        self.launch_sim_btn = QPushButton("Launch and View Simulation")
        self.launch_sim_btn.setMinimumHeight(50)
        self.launch_sim_btn.clicked.connect(self.launch_run_sim)
        self.main_layout.addWidget(self.launch_sim_btn)

        launch_suboptions_layout = QHBoxLayout()
        self.launch_sim_only_btn = QPushButton("Run Simulation Only")
        self.launch_sim_only_btn.setMinimumHeight(40)
        self.launch_sim_only_btn.clicked.connect(self.launch_run_sim_only)
        self.main_layout.addWidget(self.launch_sim_only_btn)
        launch_suboptions_layout.addWidget(self.launch_sim_only_btn)
        self.launch_vis_btn = QPushButton("Load Visualization")
        self.launch_vis_btn.setMinimumHeight(40)
        self.launch_vis_btn.clicked.connect(self.launch_view_sim)
        self.main_layout.addWidget(self.launch_vis_btn)
        launch_suboptions_layout.addWidget(self.launch_vis_btn)
        self.main_layout.addLayout(launch_suboptions_layout)

    def remove_selected_body(self):
        selection = self.body_table_view.selectionModel().currentIndex()
        if selection.isValid():
            self.body_table_model.remove_body(selection.row())

    def pack_simulation_parameters(self) -> SimulationParameters:
        return SimulationParameters(
            g_constant=self.g_input.value(),
            time_step=self.dt_input.value(),
            num_steps=self.steps_input.value(),
            softening_factor=self.softening_input.value(),
            theta=self.theta_input.value(),
            gravity=self.gravity_input.currentText(),  # type: ignore
            integrator=self.integrator_input.currentText(),  # type: ignore
        )

    def unpack_simulation_parameters(self, sim_params: SimulationParameters) -> None:
        self.g_input.setValue(sim_params.g_constant)
        self.dt_input.setValue(sim_params.time_step)
        self.steps_input.setValue(sim_params.num_steps)
        self.softening_input.setValue(sim_params.softening_factor)
        self.theta_input.setValue(sim_params.theta)
        self.gravity_input.setCurrentText(sim_params.gravity)
        self.integrator_input.setCurrentText(sim_params.integrator)

    def pack_visualizer_config(self) -> VisualizerConfig:
        return VisualizerConfig(
            step_rate=self.step_rate_input.value(),
            enable_trails=self.enable_trails_input.isChecked(),
            trail_window=self.trail_window_input.value(),
            camera_mode=self.camera_mode_input.currentText(),  # type: ignore
            spherical=self.spherical_input.isChecked(),
            default_radius=self.default_radius_input.value(),
            enable_legend=self.enable_legend_input.isChecked(),
        )

    def unpack_visualizer_config(self, visualizer_config: VisualizerConfig) -> None:
        self.step_rate_input.setValue(visualizer_config.step_rate)
        self.enable_trails_input.setChecked(visualizer_config.enable_trails)
        self.trail_window_input.setValue(visualizer_config.trail_window)
        self.camera_mode_input.setCurrentText(visualizer_config.camera_mode)
        self.spherical_input.setChecked(visualizer_config.spherical)
        self.default_radius_input.setValue(visualizer_config.default_radius)
        self.enable_legend_input.setChecked(visualizer_config.enable_legend)

    def update_bodies(self, bodies: list[BodyConfig]):
        self.body_table_model.beginResetModel()
        self.body_table_model.bodies = bodies
        self.body_table_model.endResetModel()

    def handle_generate(self):
        generator_type = self.generator_combo.currentText()
        n = self.generator_n_input.value()
        r = self.generator_r_input.value()
        bodies = generators.generators[generator_type](n, radius=r)
        self.update_bodies(bodies)

    def handle_save(self):
        csv_path = self.show_csv_file_save_dialog("Save Initial Conditions")
        if not csv_path:
            return
        if csv_path.suffix != ".csv":
            csv_path = csv_path.with_suffix(".csv")
        json_path = csv_path.parent / CONFIG_FILENAME
        sim_params = self.pack_simulation_parameters()
        visualizer_config = self.pack_visualizer_config()

        try:
            storage.save_scenario(
                sim_params,
                visualizer_config,
                self.body_table_model.bodies,
                csv_path,
                json_path,
            )
            print(
                f"Scenario saved successfully:\n\tinitial conditions: {csv_path}\n\tconfig: {json_path}"
            )
        except Exception as e:
            self.error.emit(f"Save failed: {str(e)}")

    def handle_load(self):
        csv_path = self.show_csv_file_open_dialog("Open Initial Conditions")
        if not csv_path:
            return
        json_path = csv_path.parent / CONFIG_FILENAME

        sim_params = None
        visualizer_config = None
        new_bodies = []
        try:
            sim_params, visualizer_config, new_bodies = storage.load_scenario(
                csv_path, json_path
            )
            print(
                f"Scenario loaded successfully:\n\tinitial conditions: {csv_path}\n\tconfig: {json_path}"
            )
        except Exception as e:
            self.error.emit(f"Load failed: {str(e)}")

        if sim_params:
            self.unpack_simulation_parameters(sim_params)
        if visualizer_config:
            self.unpack_visualizer_config(visualizer_config)
        self.update_bodies(new_bodies)

    def launch_run_sim(self):
        path = self.generate_run_directory_path()
        sim_parameters = self.pack_simulation_parameters()
        visualizer_config = self.pack_visualizer_config()
        self.run_sim.emit(
            path,
            sim_parameters,
            self.body_table_model.bodies,
            visualizer_config,
            True,  # auto-run the visualizer afterwards
        )

    def launch_run_sim_only(self):
        path = self.show_directory_select_dialog("Select location to run simulation")
        if not path:
            return
        sim_parameters = self.pack_simulation_parameters()
        visualizer_config = self.pack_visualizer_config()
        self.run_sim.emit(
            path,
            sim_parameters,
            self.body_table_model.bodies,
            visualizer_config,
            False,  # do not auto-run the visualizer afterwards
        )

    def launch_view_sim(self):
        path = self.show_directory_select_dialog(
            "Select simulation output file to view"
        )
        if not path:
            return
        self.view_sim.emit(path)

    def generate_run_directory_path(self) -> Path:
        run_id = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
        run_dir = self.RUN_DIR_PATH / f"run_{run_id}"
        run_dir.mkdir(parents=True, exist_ok=False)
        return run_dir

    def show_directory_select_dialog(self, caption) -> Path | None:
        path_str = QFileDialog.getExistingDirectory(
            self, caption, str(self.RUN_DIR_PATH)
        )
        return Path(path_str) if path_str else None

    def show_csv_file_save_dialog(self, caption) -> Path | None:
        path_str, _ = QFileDialog.getSaveFileName(
            self, caption, "", "CSV Files (*.csv)"
        )
        return Path(path_str) if path_str else None

    def show_csv_file_open_dialog(self, caption) -> Path | None:
        path_str, _ = QFileDialog.getOpenFileName(
            self, caption, "", "CSV Files (*.csv)"
        )
        return Path(path_str) if path_str else None
