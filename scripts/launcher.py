import sys, subprocess
from pathlib import Path
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QTableView, QFileDialog,
    QDoubleSpinBox, QSpinBox, QFormLayout, QHeaderView, QTextEdit)
from PySide6.QtCore import Qt, Signal, QAbstractTableModel, QModelIndex
from data import BodyConfig, SimulationParameters, VisualizerConfig, save_scenario, load_scenario

# table model for body configurations in the launcher's table view
class BodyTableModel(QAbstractTableModel):
    def __init__(self, bodies: list[BodyConfig] | None = None):
        super().__init__()
        self.bodies = bodies or [BodyConfig.default(1)]
        self.keys = list(BodyConfig.model_fields.keys())
        self.headers = [k.replace('_', ' ').title() for k in self.keys]

    def rowCount(self, parent=QModelIndex()):
        return len(self.bodies)

    def columnCount(self, parent=QModelIndex()):
        return len(self.headers)

    def data(self, index, role=Qt.ItemDataRole.DisplayRole):
        if not index.isValid():
            return None
        
        body = self.bodies[index.row()]
        key = self.keys[index.column()]
        if role in (Qt.ItemDataRole.DisplayRole, Qt.ItemDataRole.EditRole):
            return getattr(body, key)
        return None

    def setData(self, index, value, role=Qt.ItemDataRole.EditRole):
        if not index.isValid():
            return False
        if role != Qt.ItemDataRole.EditRole:
            return False
        
        key = self.keys[index.column()]
        try:
            if key not in ["name", "color"]:
                value = float(value)
            setattr(self.bodies[index.row()], key, value)
            self.dataChanged.emit(index, index)
            return True
        except ValueError:
            return False

    def headerData(self, section, orientation, role):
        if role != Qt.ItemDataRole.DisplayRole:
            return None
        if orientation == Qt.Orientation.Horizontal:
            return self.headers[section]
        if orientation == Qt.Orientation.Vertical:
            return str(section + 1)
        return None

    def flags(self, index):
        return Qt.ItemFlag.ItemIsEditable | Qt.ItemFlag.ItemIsEnabled | Qt.ItemFlag.ItemIsSelectable

    def add_body(self):
        self.beginInsertRows(QModelIndex(), self.rowCount(), self.rowCount())
        self.bodies.append(BodyConfig.default(len(self.bodies) + 1))
        self.endInsertRows()

    def remove_body(self, row):
        if 0 <= row < len(self.bodies):
            self.beginRemoveRows(QModelIndex(), row, row)
            self.bodies.pop(row)
            self.endRemoveRows()
            return True
        return False

# main menu for configuring, launching, and viewing a simulation
class Launcher(QWidget):

    # signal contains paths to (output CSV, config JSON)
    sim_complete = Signal(str, str)

    # signal contains error message
    sim_error = Signal(str)

    def __init__(self):
        super().__init__()
        self.initialize_ui()
        self.print_status_info("Launcher started")

    def initialize_ui(self):
        self.setWindowTitle("N-Body Launcher")
        self.setMinimumSize(600, 400)

        layout = QVBoxLayout()
        self.setLayout(layout)

        params_group = QHBoxLayout()
        form_layout = QFormLayout()

        # gravitational Constant (G)
        self.g_input = QDoubleSpinBox()
        self.g_input.setDecimals(15) # High precision for G
        self.g_input.setRange(0, 1)
        self.g_input.setValue(6.67430e-11)
        form_layout.addRow("G Constant:", self.g_input)

        # time Step (t)
        self.dt_input = QDoubleSpinBox()
        self.dt_input.setDecimals(4)
        self.dt_input.setRange(0.0001, 100000)
        self.dt_input.setValue(0.1)
        form_layout.addRow("Time Step (s):", self.dt_input)

        # number of Steps (n)
        self.steps_input = QSpinBox()
        self.steps_input.setRange(1, 10000000)
        self.steps_input.setSingleStep(1000)
        self.steps_input.setValue(10000)
        form_layout.addRow("Num Steps:", self.steps_input)


        # softening Factor
        self.softening_input = QDoubleSpinBox()
        self.softening_input.setDecimals(6)
        self.softening_input.setRange(0, 100)
        self.softening_input.setValue(0.01)
        form_layout.addRow("Softening Factor:", self.softening_input)

        params_group.addLayout(form_layout)
        params_group.addStretch(1)
        
        # Status message display
        self.status_box = QTextEdit()
        self.status_box.setReadOnly(True)
        self.status_box.setMaximumHeight(100)
        self.status_box.setLineWrapMode(QTextEdit.NoWrap) # type: ignore
        params_group.addWidget(self.status_box, 3)
        
        layout.addLayout(params_group)

        # table of body configurations
        self.model = BodyTableModel()
        self.table_view = QTableView()
        self.table_view.setModel(self.model)
        self.table_view.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch) # type: ignore
        layout.addWidget(self.table_view)

        # Button controls layout
        button_layout = QHBoxLayout()
        self.add_btn = QPushButton("Add Body")
        self.add_btn.clicked.connect(self.model.add_body)
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
        layout.addLayout(button_layout)

        # launch button
        self.launch_sim_btn = QPushButton("Launch Simulation")
        self.launch_sim_btn.setMinimumHeight(50)
        self.launch_sim_btn.clicked.connect(self.launch_sim)
        layout.addWidget(self.launch_sim_btn)

    def remove_selected_body(self):
        selection = self.table_view.selectionModel().currentIndex()
        if selection.isValid():
            self.model.remove_body(selection.row())

    def print_status_info(self, message):
        self.status_box.append(f"[INFO] {message}")

    def print_status_warn(self, message):
        self.status_box.append(f"[WARN] {message}")

    def print_status_error(self, message):
        self.status_box.append(f"[ERROR] {message}")

    def handle_save(self):
        sim_params = SimulationParameters(
            g_constant=self.g_input.value(),
            time_step=self.dt_input.value(),
            num_steps=self.steps_input.value(),
            softening_factor=self.softening_input.value()
        )
        visualizer_config = VisualizerConfig()

        path_str, _ = QFileDialog.getSaveFileName(
            self,
            "Save Initial Conditions",
            "",
            "CSV Files (*.csv)"
        )
        if path_str:
            csv_path = Path(path_str)
            json_path = csv_path.with_suffix(".json")
            try:
                save_scenario(sim_params, visualizer_config, self.model.bodies, csv_path, json_path)
                self.print_status_info(f"Successfully saved:\n{csv_path}\n{json_path}")
            except Exception as e:
                self.sim_error.emit(f"Save failed: {str(e)}")

    def handle_load(self):
        path_str, _ = QFileDialog.getOpenFileName(
            self,
            "Open Initial Conditions",
            "",
            "CSV Files (*.csv)"
        )

        if path_str:
            csv_path = Path(path_str)
            json_path = csv_path.with_suffix(".json")

            # json configuration is optional
            if not json_path.exists():
                json_path = None
            
            try:
                sim_params, visualizer_config, new_bodies = load_scenario(csv_path, json_path)
                self.model.beginResetModel()
                self.model.bodies = new_bodies
                self.model.endResetModel()
                
                # Update simulation parameters if they exist
                if sim_params:
                    self.g_input.setValue(sim_params.g_constant)
                    self.dt_input.setValue(sim_params.time_step)
                    self.steps_input.setValue(sim_params.num_steps)
                    self.softening_input.setValue(sim_params.softening_factor)

                #TODO add visualizer options here
                
                self.print_status_info(f"Successfully loaded: {csv_path}")
            except Exception as e:
                self.print_status_error(f"Load failed: {str(e)}")


    def launch_sim(self):
        self.print_status_info("Preparing simulation...")

        RUN_DIR_PATH = Path("data/run")
        RUN_DIR_PATH.mkdir(parents=True, exist_ok=True)

        IC_PATH = RUN_DIR_PATH / "sim.csv"
        CONFIG_PATH = RUN_DIR_PATH / "sim.json"
        OUTPUT_PATH = RUN_DIR_PATH / "output.csv"

        if sys.platform == "win32":
            BIN_PATH = Path("n-body-sim.exe")
        else:
            BIN_PATH = Path("n_body_sim_bin").resolve()

        if not BIN_PATH.is_file():
            self.sim_error.emit(f"Physics engine binary not found at: {BIN_PATH}")
            return
        
        try:
            sim_params = SimulationParameters(
                g_constant=self.g_input.value(),
                time_step=self.dt_input.value(),
                num_steps=self.steps_input.value(),
                softening_factor=self.softening_input.value()
            )
            visualizer_config = VisualizerConfig()
            save_scenario(sim_params, visualizer_config, self.model.bodies, IC_PATH, CONFIG_PATH)
            
            command = [
                str(BIN_PATH),
                "-i", str(IC_PATH),
                "-o", str(OUTPUT_PATH),
                "-g", str(self.g_input.value()),
                "-t", str(self.dt_input.value()),
                "-n", str(self.steps_input.value()),
                "--softening-factor", str(self.softening_input.value())
            ]

            self.print_status_info(f"Launching simulation: {' '.join(command)}")
            result = subprocess.run(command, check=True, text=True, capture_output=True)
            self.print_status_info("Simulation finished")

            self.sim_complete.emit(str(OUTPUT_PATH), str(CONFIG_PATH))
        except subprocess.CalledProcessError as e:
            self.sim_error.emit(f"Physics Engine Error:\n{e.stderr}")
        except Exception as e:
            self.sim_error.emit(f"Error:\n{str(e)}")
