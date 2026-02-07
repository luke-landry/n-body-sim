from PySide6.QtWidgets import QApplication, QMessageBox
from launcher import Launcher
from visualizer import Visualizer
from runners import Runner, SimulationRunner, VisualizerRunner
from data import BodyConfig, SimulationParameters, VisualizerConfig
from pathlib import Path
import storage


class Controller:
    def __init__(self) -> None:
        self.app = QApplication()
        self.runners: list[Runner] = []

        self.launcher = Launcher()
        self.launcher.run_sim.connect(self.run_sim)
        self.launcher.view_sim.connect(self.view_sim)
        self.launcher.error.connect(self.show_error_dialog)

    def run(self):
        self.launcher.show()
        self.app.exec()

    def run_sim(self,
            path: Path,                   # directory path
            sim_parameters: SimulationParameters,   # simulation parameters
            bodies: list[BodyConfig],       # body configs
            visualizer_config: VisualizerConfig,       # visualizer config
            run_visualizer: bool                    # auto-run the visualizer afterwards
        ):
        ic_path = path / "sim.csv"
        config_path = path / "sim.json"
        output_path = path / "output.csv"

        try:
            storage.save_scenario(sim_parameters, visualizer_config, bodies, ic_path, config_path)
        except Exception as e:
            self.show_error_dialog(f"Failed to save scenario data: {e}")
            return
        
        runner = SimulationRunner(ic_path, output_path, sim_parameters)
        runner.finished.connect(
            lambda r=runner, v=run_visualizer, p=path: self.handle_simulation_finished(r, v, p)      
        )
        runner.error.connect(self.on_simulation_error)
        self.runners.append(runner)
        runner.start()

    def handle_simulation_finished(self, runner, run_visualizer, path: Path):
        if runner in self.runners:
            self.runners.remove(runner)
        runner.deleteLater()

        if run_visualizer:
            self.view_sim(path)


    def view_sim(self, path: Path):
        config_path = path / "sim.json"
        output_path = path / "output.csv" if path.is_dir() else path
        runner = VisualizerRunner(output_path, config_path)
        runner.finished.connect(lambda r=runner: self.handle_visualizer_finished(r))
        runner.error.connect(self.on_visualizer_error)
        self.runners.append(runner)
        runner.start()
    
    def handle_visualizer_finished(self, runner):
        if runner in self.runners:
            self.runners.remove(runner)
        runner.deleteLater()
    
    def on_simulation_error(self, message: str):
        self.show_error_dialog(message)

    def on_visualizer_error(self, message: str):
        self.show_error_dialog(message)


    def show_error_dialog(self, message):
        print(f"Error: {message}")
        msg_box = QMessageBox()
        msg_box.setIcon(QMessageBox.Icon.Critical)
        msg_box.setWindowTitle("Simulation Error")
        msg_box.setText(message)
        msg_box.setStandardButtons(QMessageBox.StandardButton.Ok)
        msg_box.exec()
