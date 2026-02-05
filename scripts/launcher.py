import sys
import uuid
from pathlib import Path
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QTableView, QFileDialog,
    QDoubleSpinBox, QSpinBox, QFormLayout, QHeaderView, QComboBox, QCheckBox, QGroupBox, QMessageBox)
from data import BodyConfig, SimulationParameters, VisualizerConfig
from models import BodyTableModel
from runner import Runner
import storage
import generators


# main menu for configuring, launching, and viewing a simulation
class Launcher(QWidget):
    def __init__(self):
        super().__init__()

        BASE_PATH = Path(__file__).parents[1]

        self.RUN_DIR_PATH = BASE_PATH / Path("data/run")
        self.RUN_DIR_PATH.mkdir(parents=True, exist_ok=True)
        self.BIN_PATH = BASE_PATH / "bin" / ("n-body-sim.exe" if sys.platform == "win32" else "n_body_sim_bin")

        self.runners: list[Runner] = []

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
        self.g_input.setDecimals(15) # High precision for G
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
        self.gravity_input.addItems(["newton"])
        self.gravity_input.setCurrentText(default_sim.gravity)
        sim_form_layout.addRow("Gravity Method:", self.gravity_input)

        # integrator method
        self.integrator_input = QComboBox()
        self.integrator_input.addItems(["euler"])
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
        self.body_table_view.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch) # type: ignore
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
        self.generator_combo.addItems(generators.GENERATOR_NAMES)
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
        self.generate_btn = QPushButton("Generate Random Scenario")
        self.generate_btn.clicked.connect(self.handle_generate)
        generator_layout.addWidget(self.generate_btn)
        self.main_layout.addLayout(generator_layout)

        # launch button
        self.launch_sim_btn = QPushButton("Launch Simulation")
        self.launch_sim_btn.setMinimumHeight(50)
        self.launch_sim_btn.clicked.connect(self.launch_sim)
        self.main_layout.addWidget(self.launch_sim_btn)

    def remove_selected_body(self):
        selection = self.body_table_view.selectionModel().currentIndex()
        if selection.isValid():
            self.body_table_model.remove_body(selection.row())

    def build_simulation_parameters(self) -> SimulationParameters:
        return SimulationParameters(
            g_constant=self.g_input.value(),
            time_step=self.dt_input.value(),
            num_steps=self.steps_input.value(),
            softening_factor=self.softening_input.value(),
            theta=self.theta_input.value(),
            gravity=self.gravity_input.currentText(), # type: ignore
            integrator=self.integrator_input.currentText() # type: ignore
        )

    def build_visualizer_config(self) -> VisualizerConfig:
        return VisualizerConfig(
            step_rate=self.step_rate_input.value(),
            enable_trails=self.enable_trails_input.isChecked(),
            trail_window=self.trail_window_input.value(),
            camera_mode=self.camera_mode_input.currentText(), # type: ignore
            spherical=self.spherical_input.isChecked(),
            default_radius=self.default_radius_input.value(),
            enable_legend=self.enable_legend_input.isChecked()
        )

    def update_bodies(self, bodies: list[BodyConfig]):
        self.body_table_model.beginResetModel()
        self.body_table_model.bodies = bodies
        self.body_table_model.endResetModel()

    def handle_generate(self):
        try:
            n = self.generator_n_input.value()
            r = self.generator_r_input.value()
            generator_type = self.generator_combo.currentText()
            
            if generator_type == "Star System":
                bodies = generators.generate_single_star_system(n, radius=r)
            else:
                print(f"Unknown generator type: {generator_type}")
                return
            
            self.update_bodies(bodies)
            print(f"Generated {len(bodies)} bodies using {generator_type}")
        except Exception as e:
            self.show_error_dialog(f"Generation failed: {str(e)}")

    def handle_save(self):
        sim_params = self.build_simulation_parameters()
        visualizer_config = self.build_visualizer_config()

        path_str, _ = QFileDialog.getSaveFileName(
            self,
            "Save Initial Conditions",
            "",
            "CSV Files (*.csv)"
        )

        if not path_str:
            return

        csv_path = Path(path_str)
        json_path = csv_path.with_suffix(".json")
        try:
            storage.save_scenario(sim_params, visualizer_config, self.body_table_model.bodies, csv_path, json_path)
            print(f"Successfully saved:\n{csv_path}\n{json_path}")
        except Exception as e:
            self.show_error_dialog(f"Save failed: {str(e)}")

    def handle_load(self):
        path_str, _ = QFileDialog.getOpenFileName(
            self,
            "Open Initial Conditions",
            "",
            "CSV Files (*.csv)"
        )

        if not path_str:
            return
        
        csv_path = Path(path_str)
        json_path = csv_path.with_suffix(".json")

        # json configuration is optional
        if not json_path.exists():
            json_path = None
        
        try:
            sim_params, visualizer_config, new_bodies = storage.load_scenario(csv_path, json_path)
            self.body_table_model.beginResetModel()
            self.body_table_model.bodies = new_bodies
            self.body_table_model.endResetModel()
            
            # Update simulation parameters if they exist
            if sim_params:
                self.g_input.setValue(sim_params.g_constant)
                self.dt_input.setValue(sim_params.time_step)
                self.steps_input.setValue(sim_params.num_steps)
                self.softening_input.setValue(sim_params.softening_factor)
                self.theta_input.setValue(sim_params.theta)
                self.gravity_input.setCurrentText(sim_params.gravity)
                self.integrator_input.setCurrentText(sim_params.integrator)

            # Update visualizer configuration if it exists
            if visualizer_config:
                self.step_rate_input.setValue(visualizer_config.step_rate)
                self.enable_trails_input.setChecked(visualizer_config.enable_trails)
                self.trail_window_input.setValue(visualizer_config.trail_window)
                self.camera_mode_input.setCurrentText(visualizer_config.camera_mode)
                self.spherical_input.setChecked(visualizer_config.spherical)
                self.default_radius_input.setValue(visualizer_config.default_radius)
                self.enable_legend_input.setChecked(visualizer_config.enable_legend)
            
            print(f"Successfully loaded: {csv_path}")
        except Exception as e:
            self.show_error_dialog(f"Load failed: {str(e)}")

    def launch_sim(self):
        print("Preparing simulation...")

        run_id = uuid.uuid4().hex[:8]

        run_dir = self.RUN_DIR_PATH / f"run_{run_id}"
        run_dir.mkdir(parents=True, exist_ok=False)

        ic_path = run_dir / "sim.csv"
        config_path = run_dir / "sim.json"
        output_path = run_dir / "output.csv"

        if not self.BIN_PATH.is_file():
            self.show_error_dialog(f"Physics engine binary not found at: {self.BIN_PATH}")
            return
        
        try:
            sim_params = self.build_simulation_parameters()
            visualizer_config = self.build_visualizer_config()
            storage.save_scenario(
                sim_params,
                visualizer_config,
                self.body_table_model.bodies,
                ic_path,
                config_path)
            
            args = [
                "-i", str(ic_path),
                "-o", str(output_path),
                "-g", str(self.g_input.value()),
                "-t", str(self.dt_input.value()),
                "-n", str(self.steps_input.value()),
                "--softening-factor", str(self.softening_input.value()),
                "--theta", str(self.theta_input.value()),
                "--gravity", self.gravity_input.currentText(),
                "--integrator", self.integrator_input.currentText()
            ]

            print(f"Launching simulation: {self.BIN_PATH} {' '.join(args)}")
            runner = Runner(
                parent=self,
                bin_path=self.BIN_PATH,
                ic_path=ic_path,
                output_path=output_path,
                config_path=config_path,
                sim_args=args,
            )

            runner.error.connect(self.show_error_dialog)
            runner.finished.connect(lambda r=runner: self.runners.remove(r))
            self.runners.append(runner)
            runner.start()
        except Exception as e:
            self.show_error_dialog(f"Error:\n{str(e)}")
            import traceback
            traceback.print_exc()

    def show_error_dialog(self, error_message):
        print(f"Error: {error_message}")
        msg_box = QMessageBox(self)
        msg_box.setIcon(QMessageBox.Icon.Critical)
        msg_box.setWindowTitle("Simulation Error")
        msg_box.setText(error_message)
        msg_box.setStandardButtons(QMessageBox.StandardButton.Ok)
        msg_box.exec()