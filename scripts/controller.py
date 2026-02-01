
from PySide6.QtWidgets import QApplication, QMessageBox
from pathlib import Path
from launcher import Launcher
from visualizer import Visualizer

class Controller:
    def __init__(self):
        self.app = QApplication()
        self.launcher = Launcher()
        self.launcher.sim_complete.connect(self.on_sim_complete)
        self.launcher.sim_error.connect(self.on_sim_error)

    def on_sim_complete(self, data_path_str: str, config_path_str: str | None):
        try:
            data_path = Path(data_path_str)
            config_path = Path(config_path_str) if config_path_str else None
            self.visualizer = Visualizer.from_paths(data_path, config_path)
            self.visualizer.show()
        except Exception as e:
            self.show_error_dialog(f"Error during visualization:\n{e}")

    def on_sim_error(self, error_message):
        print(f"Error during sim: {error_message}")
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
