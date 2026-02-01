import csv, json
import pandas as pd
import subprocess
import sys, os
from pathlib import Path
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QTableView, QFileDialog, QLineEdit, QLabel, 
    QDoubleSpinBox, QSpinBox, QFormLayout, QHeaderView, QTextEdit)
from PySide6.QtCore import Qt, Signal, QAbstractTableModel, QModelIndex

# model for body configurations in the launcher table view
class BodyTableModel(QAbstractTableModel):
    def __init__(self, bodies=None):
        super().__init__()
        self.bodies = bodies or [
            {"name": "Sun", "mass": 1000.0, "radius": 0.5, "color": "yellow", 
             "pos_x": 0.0, "pos_y": 0.0, "pos_z": 0.0, "vel_x": 0.0, "vel_y": 0.0, "vel_z": 0.0}
        ]
        self.headers = ["Name", "Color", "Radius", "Mass", "Pos X", "Pos Y", "Pos Z", "Vel X", "Vel Y", "Vel Z"]
        self.keys = ["name", "color", "radius", "mass", "pos_x", "pos_y", "pos_z", "vel_x", "vel_y", "vel_z"]

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
            return body[key]
        return None

    def setData(self, index, value, role=Qt.ItemDataRole.EditRole):
        if index.isValid() and role == Qt.ItemDataRole.EditRole:
            key = self.keys[index.column()]
            try:
                if key not in ["name", "color"]:
                    value = float(value)
                self.bodies[index.row()][key] = value
                self.dataChanged.emit(index, index)
                return True
            except ValueError:
                return False
        return False

    def headerData(self, section, orientation, role):
        if role == Qt.ItemDataRole.DisplayRole:
            if orientation == Qt.Orientation.Horizontal:
                return self.headers[section]
            if orientation == Qt.Orientation.Vertical:
                return str(section + 1)
        return None

    def flags(self, index):
        return Qt.ItemFlag.ItemIsEditable | Qt.ItemFlag.ItemIsEnabled | Qt.ItemFlag.ItemIsSelectable

    def add_body(self):
        self.beginInsertRows(QModelIndex(), self.rowCount(), self.rowCount())
        self.bodies.append({k: (0.0 if k not in ["name", "color"] else "") for k in self.keys})
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
        self.status_box.setLineWrapMode(QTextEdit.NoWrap)
        params_group.addWidget(self.status_box, 3)
        
        layout.addLayout(params_group)

        # table of body configurations
        self.model = BodyTableModel()
        self.table_view = QTableView()
        self.table_view.setModel(self.model)
        self.table_view.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)
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
                self.save_simulation_files(self.model.bodies, csv_path, json_path)
                self.print_status_info(f"Successfully saved:\n{csv_path}\n{json_path}")
            except Exception as e:
                self.sim_error.emit(f"Save failed: {str(e)}")

    def save_simulation_files(self, bodies, ic_csv_path: Path, json_path: Path):
        csv_keys = ["mass", "pos_x", "pos_y", "pos_z", "vel_x", "vel_y", "vel_z"]
        config = {
            "simulationParams": {
                "g_constant": self.g_input.value(),
                "time_step": self.dt_input.value(),
                "num_steps": self.steps_input.value(),
                "softening_factor": self.softening_input.value()
            },
            "visualizerConfig": {
                "names": [],
                "radii": [],
                "colors": []
            }
        }

        # write initial conditions csv
        with open(ic_csv_path, 'w', newline='') as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=csv_keys, extrasaction='ignore')
            writer.writeheader()
            for body in bodies:
                writer.writerow(body)
                config["visualizerConfig"]["names"].append(body["name"])
                config["visualizerConfig"]["radii"].append(body["radius"])
                config["visualizerConfig"]["colors"].append(body["color"])

        # write json
        with open(json_path, 'w') as jsonfile:
            json.dump(config, jsonfile, indent=4)

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
            
            try:
                new_bodies = self.load_simulation_files(csv_path, json_path)
                self.model.beginResetModel()
                self.model.bodies = new_bodies
                self.model.endResetModel()
                self.print_status_info(f"Successfully loaded: {csv_path}")
            except Exception as e:
                self.sim_error.emit(f"Load failed: {str(e)}")

    def load_simulation_files(self, ic_csv_path: Path, json_path: Path):

        # read initial conditions csv
        df = pd.read_csv(ic_csv_path)
        numeric_keys = ["mass", "pos_x", "pos_y", "pos_z", "vel_x", "vel_y", "vel_z"]
        df = df.reindex(columns=numeric_keys).fillna(0.0)

        missing_cols = [col for col in numeric_keys if col not in df.columns]
        if missing_cols:
            raise ValueError(f"CSV is missing required columns: {', '.join(missing_cols)}")
        if df[numeric_keys].isnull().values.any():
            raise ValueError("CSV contains empty or malformed numeric cells.")

        # read config json
        config = {}
        if json_path.exists():
            with open(json_path, 'r') as f:
                config = json.load(f)
        elif str(json_path):
            self.print_status_warn(f"No configuration file found at {json_path}")
        
        # combine initial confitions and configs back into a
        # list of dictionaries for the table data model
        bodies = []
        num_rows = len(df)
        visualizer_config = config.get("visualizerConfig", config) if isinstance(config, dict) else {}
        simulation_params = config.get("simulationParams", {}) if isinstance(config, dict) else {}

        if isinstance(simulation_params, dict):
            if "g_constant" in simulation_params:
                self.g_input.setValue(simulation_params["g_constant"])
            if "time_step" in simulation_params:
                self.dt_input.setValue(simulation_params["time_step"])
            if "num_steps" in simulation_params:
                self.steps_input.setValue(int(simulation_params["num_steps"]))
            if "softening_factor" in simulation_params:
                self.softening_input.setValue(simulation_params["softening_factor"])

        names = visualizer_config.get("names", []) if isinstance(visualizer_config, dict) else []
        radii = visualizer_config.get("radii", []) if isinstance(visualizer_config, dict) else []
        colors = visualizer_config.get("colors", []) if isinstance(visualizer_config, dict) else []

        for i, row_dict in enumerate(df.to_dict('records')):
            row_dict["name"] = names[i] if i < len(names) else f"Body {i+1}"
            row_dict["radius"] = radii[i] if i < len(radii) else 0.1
            row_dict["color"] = colors[i] if i < len(colors) else "#ffffff"
            bodies.append(row_dict)
        
        return bodies

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
            self.save_simulation_files(self.model.bodies, IC_PATH, CONFIG_PATH)
            
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

        

