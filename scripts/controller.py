
from PySide6.QtWidgets import QApplication, QMessageBox, QProgressDialog
from PySide6.QtCore import Qt
from pathlib import Path
from launcher import Launcher
import storage

class Controller:
    def __init__(self):
        self.app = QApplication()
        self.launcher = Launcher()
        self.loading_dialog = None
        self.launcher.sim_complete.connect(self.on_sim_complete)
        self.launcher.sim_error.connect(self.on_sim_error)
        self.launcher.sim_started.connect(self.on_sim_started)

    def on_sim_started(self):
        if self.loading_dialog is None:
            self.loading_dialog = QProgressDialog("Running simulation...", "", 0, 0, self.launcher)
            self.loading_dialog.setWindowTitle("Simulation In Progress")
            self.loading_dialog.setWindowModality(Qt.WindowModality.ApplicationModal)
            self.loading_dialog.setCancelButton(None)
            self.loading_dialog.setMinimumDuration(0)
        self.loading_dialog.show()

    def on_sim_complete(self, data_path_str: str, config_path_str: str | None):
        self.close_loading_dialog()
        try:
            data_path = Path(data_path_str)
            config_path = Path(config_path_str) if config_path_str else None
            self.visualizer = storage.create_visualizer_from_paths(data_path, config_path)
            self.visualizer.show()
        except Exception as e:
            self.show_error_dialog(f"Error during visualization:\n{e}")

    def on_sim_error(self, error_message):
        self.close_loading_dialog()
        print(f"Error during sim: {error_message}")
        self.show_error_dialog(error_message)
        
    def show_error_dialog(self, error_message):
        msg_box = QMessageBox(self.launcher)
        msg_box.setIcon(QMessageBox.Icon.Critical)
        msg_box.setWindowTitle("Simulation Error")
        msg_box.setText(error_message)
        msg_box.setStandardButtons(QMessageBox.StandardButton.Ok)
        msg_box.exec()

    def close_loading_dialog(self):
        if self.loading_dialog is not None:
            self.loading_dialog.hide()

    def run(self):
        self.launcher.show()
        return self.app.exec()
