from pathlib import Path
import sys
from PySide6.QtWidgets import QProgressDialog
from PySide6.QtCore import Qt, QObject, QProcess, QThread, Signal
import storage
from visualizer import Visualizer
from data import SimulationData, VisualizerConfig, SimulationParameters

# Base runner class
class Runner(QObject):
    finished = Signal()
    error = Signal(str)     # error message

    def __init__(self):
        super().__init__()

    def start(self):
        raise NotImplementedError("start() must be implemented by subclasses")

# Simulation runner class to handle running the simulation process
class SimulationRunner(Runner):
    def __init__(self, ic_path: Path, output_path: Path, sim_parameters: SimulationParameters) -> None:
        super().__init__()
        self.ic_path = ic_path
        self.output_path = output_path
        self.sim_parameters = sim_parameters
        self._loading_dialog: QProgressDialog | None = None

        BASE_PATH = Path(__file__).parents[1]
        self.BIN_PATH = BASE_PATH / "bin" / ("n-body-sim.exe" if sys.platform == "win32" else "n-body-sim")
        self.RUN_DIR_PATH = BASE_PATH / Path("data/run")
        self.RUN_DIR_PATH.mkdir(parents=True, exist_ok=True)


    def start(self):
        if not self.ic_path.is_file():
            self.error.emit(f"Initial conditions file not found: {self.ic_path}")
            return
        if not self.BIN_PATH.is_file():
            self.error.emit(f"Binary not found: {self.BIN_PATH}")
            return
        self.start_sim_process()

    def build_sim_args(self) -> list[str]:
        return [
                "-i", str(self.ic_path),
                "-o", str(self.output_path),
                "-g", str(self.sim_parameters.g_constant),
                "-t", str(self.sim_parameters.time_step),
                "-n", str(self.sim_parameters.num_steps),
                "--softening-factor", str(self.sim_parameters.softening_factor),
                "--theta", str(self.sim_parameters.theta),
                "--gravity", self.sim_parameters.gravity,
                "--integrator", self.sim_parameters.integrator
        ]

    def start_sim_process(self):
        self.sim_process = QProcess(self)
        self.sim_process.setProgram(str(self.BIN_PATH))
        self.sim_process.setArguments(self.build_sim_args())
        self.sim_process.setProcessChannelMode(QProcess.MergedChannels)  # type: ignore
        self.sim_process.finished.connect(self.on_sim_process_finished)
        self.sim_process.errorOccurred.connect(self.on_sim_process_error)
        self._loading_dialog = self.create_loading_dialog(
            title="Running Simulation",
            label="Simulation running. This may take a while..."
        )
        self.sim_process.start()
        print(f"Started simulation process with PID {self.sim_process.processId()}:\n\targuments: {self.build_sim_args()}")

    def on_sim_process_finished(self, exit_code: int, exit_status: QProcess.ExitStatus):
        self.close_loading_dialog()
        self.sim_process.deleteLater()
        if exit_status == QProcess.ExitStatus.NormalExit:
            self.finished.emit()
        else:
            self.error.emit(f"Simulation process exited with status: {exit_status}")

    def on_sim_process_error(self, error: QProcess.ProcessError):
        self.close_loading_dialog()
        self.sim_process.deleteLater()
        self.error.emit(f"Simulation process error occurred: {error.name}")

    def create_loading_dialog(self, title: str, label: str) -> QProgressDialog:
        dialog = QProgressDialog(label, None, 0, 0) # type: ignore
        dialog.setWindowTitle(title)
        dialog.setWindowModality(Qt.ApplicationModal) # type: ignore
        dialog.setAutoClose(False)
        dialog.setAutoReset(False)
        dialog.setCancelButton(None)
        dialog.show()
        return dialog

    def close_loading_dialog(self) -> None:
        if self._loading_dialog:
            self._loading_dialog.close()
            self._loading_dialog.deleteLater()
            self._loading_dialog = None


# Helper worker class to load simulation data and config
# asynchronously, since loading large datasets can take a long time
class LoadVisualizerWorker(QObject):
    finished = Signal(SimulationData, VisualizerConfig)
    error = Signal(str)

    def __init__(self, data_path: Path, config_path: Path):
        super().__init__()
        self.data_path = data_path
        self.config_path = config_path

    def run(self):
        try:
            data = storage.load_simulation_data_from_path(self.data_path)
            config = storage.load_visualizer_config_from_json(self.config_path)
            self.finished.emit(data, config)
        except Exception as e:
            self.error.emit(str(e))
        
# Visualizer runner class to handle loading the visualizer in a separate thread
class VisualizerRunner(Runner):
    def __init__(self, output_path, config_path) -> None:
        super().__init__()
        self.output_path = output_path
        self.config_path = config_path
        self._loading_dialog: QProgressDialog | None = None

    def start(self):
        if not self.output_path.is_file():
            self.error.emit(f"Simulation output not found: {self.output_path}")
            return
        if not self.config_path.is_file():
            self.error.emit(f"Visualizer config not found: {self.config_path}")
            return
        self.start_visualizer_load()

    def start_visualizer_load(self):
        self._loading_dialog = self.create_loading_dialog(
            title="Loading Visualizer",
            label="Loading simulation data..."
        )
        self._loading_thread = QThread(self)
        self._loading_worker = LoadVisualizerWorker(self.output_path,self.config_path)
        self._loading_worker.moveToThread(self._loading_thread)

        self._loading_thread.started.connect(self._loading_worker.run)
        self._loading_worker.finished.connect(self.on_visualizer_loaded)
        self._loading_worker.finished.connect(self._loading_thread.quit)
        self._loading_worker.finished.connect(self._loading_worker.deleteLater)
        self._loading_worker.error.connect(self.on_worker_error)
        self._loading_worker.error.connect(self._loading_thread.quit)
        self._loading_worker.error.connect(self._loading_worker.deleteLater)
        self._loading_thread.finished.connect(self._loading_thread.deleteLater)
        self._loading_thread.start()
        print(f"Started visualizer loading thread:\n\tdata: {self.output_path}\n\tconfig: {self.config_path}")

    def on_visualizer_loaded(self, data: SimulationData, config: VisualizerConfig):
        self.close_loading_dialog()
        self.visualizer = Visualizer(data, config)
        self.visualizer.setAttribute(Qt.WA_DeleteOnClose, True) # type: ignore
        self.visualizer.destroyed.connect(self.on_visualizer_closed)
        self.visualizer.show()

    def on_visualizer_closed(self):
        self.visualizer = None
        self.finished.emit()
        self.deleteLater()

    def on_worker_error(self, message: str):
        self.close_loading_dialog()
        self.error.emit(message)
        self.finished.emit() 
        self.deleteLater()

    def create_loading_dialog(self, title: str, label: str) -> QProgressDialog:
        dialog = QProgressDialog(label, None, 0, 0) # type: ignore
        dialog.setWindowTitle(title)
        dialog.setWindowModality(Qt.ApplicationModal) # type: ignore
        dialog.setAutoClose(False)
        dialog.setAutoReset(False)
        dialog.setCancelButton(None)
        dialog.show()
        return dialog

    def close_loading_dialog(self) -> None:
        if self._loading_dialog:
            self._loading_dialog.close()
            self._loading_dialog.deleteLater()
            self._loading_dialog = None

