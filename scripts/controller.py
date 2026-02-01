
from PySide6.QtWidgets import QApplication, QMessageBox
import os
from launcher import Launcher
from data import VisualizerConfig, load_sim_data_from_csv, load_sim_data_from_bin
from visualizer import Visualizer

class Controller:
    def __init__(self):
        self.app = QApplication()
        self.launcher = Launcher()
        self.launcher.sim_complete.connect(self.on_sim_complete)
        self.launcher.sim_error.connect(self.on_sim_error)

    def on_sim_complete(self, data_path, config_path):
        try:
            data_ext = data_path.split(".")[-1].lower()
            if data_ext == "csv":
                data = load_sim_data_from_csv(data_path)
            elif data_ext == "nbody":
                data = load_sim_data_from_bin(data_path) 
            else:
                raise ValueError(f"Unsupported data format: .{data_ext}")

            if config_path and os.path.exists(config_path):
                config = VisualizerConfig.from_json(config_path)
            else:
                config = VisualizerConfig()

            self.visualizer = Visualizer(data, config)
            self.visualizer.show()
        except Exception as e:
            self.show_error_dialog(f"Error during visualization:\n{str(e)}")

    def on_sim_error(self, error_message):
        print(f"ERROR: {error_message}")
        self.show_error_dialog(error_message)
        
    def show_error_dialog(self, error_message):
        msg_box = QMessageBox(self.launcher)
        msg_box.setIcon(QMessageBox.Icon.Critical)
        msg_box.setWindowTitle("Simulation Error")
        msg_box.setText(error_message)
        msg_box.setStandardButtons(QMessageBox.StandardButton.Ok)
        msg_box.exec()

    def run(self):
        self.launcher.show()
        return self.app.exec()
