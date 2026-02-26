from functools import partial
from pathlib import Path

import storage
from launcher import (
    CONFIG_FILENAME,
    INITIAL_CONDITIONS_FILENAME,
    OUTPUT_FILENAME,
    Launcher,
)
from PySide6.QtWidgets import QApplication, QMessageBox
from runners import Runner, SimulationRunner, VisualizerRunner
from schema import BodyConfig, SimulationParameters, VisualizerConfig


class Controller:
    def __init__(self) -> None:
        self.app = QApplication()
        self.runners: list[Runner] = []
        self.runner_metadata: dict[Runner, dict] = {}

        self.launcher: Launcher = Launcher()
        self.launcher.run_sim.connect(self.run_sim)
        self.launcher.view_sim.connect(self.view_sim)
        self.launcher.error.connect(self.show_error_dialog)

    def run(self):
        self.launcher.show()
        return self.app.exec()

    def run_sim(
        self,
        path: Path,
        sim_parameters: SimulationParameters,
        bodies: list[BodyConfig],
        visualizer_config: VisualizerConfig,
        run_visualizer: bool,
    ):
        ic_path = path / INITIAL_CONDITIONS_FILENAME
        config_path = path / CONFIG_FILENAME
        output_path = path / OUTPUT_FILENAME

        try:
            storage.save_scenario(
                sim_parameters, visualizer_config, bodies, ic_path, config_path
            )
        except Exception as e:
            self.show_error_dialog(f"Failed to save scenario data: {e}")
            return

        runner = SimulationRunner(ic_path, output_path, sim_parameters)
        runner.finished.connect(partial(self.handle_runner_finished, runner))
        runner.error.connect(self.handle_runner_error)
        self.runners.append(runner)
        self.runner_metadata[runner] = {
            "run_visualizer": run_visualizer,
            "path": path,
        }
        runner.start()

    def view_sim(self, path: Path):
        config_path = path / CONFIG_FILENAME
        output_path = path / OUTPUT_FILENAME
        runner = VisualizerRunner(output_path, config_path)
        runner.finished.connect(partial(self.handle_runner_finished, runner))
        runner.error.connect(self.handle_runner_error)
        self.runners.append(runner)
        runner.start()

    def handle_runner_finished(self, runner: Runner, exit_code: int):
        self.cleanup_runner(runner)
        meta = self.runner_metadata.pop(runner, {})
        run_visualizer = meta.get("run_visualizer", False)
        path = meta.get("path", None)
        if run_visualizer and path and exit_code == 0:
            self.view_sim(path)

    def handle_runner_error(self, message: str):
        self.show_error_dialog(message)

    def cleanup_runner(self, runner: Runner) -> None:
        if runner in self.runners:
            self.runners.remove(runner)
        runner.deleteLater()

    def show_error_dialog(self, message):
        print(f"Error: {message}")
        msg_box = QMessageBox()
        msg_box.setIcon(QMessageBox.Icon.Critical)
        msg_box.setWindowTitle("Simulation Error")
        msg_box.setText(message)
        msg_box.setStandardButtons(QMessageBox.StandardButton.Ok)
        msg_box.exec()
