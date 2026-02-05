from pathlib import Path
from PySide6.QtWidgets import QProgressDialog
from PySide6.QtCore import Qt, QObject, QProcess, QThread, Signal
import storage
import shutil
from visualizer import Visualizer
from data import SimulationData, VisualizerConfig

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


class Runner(QObject):
    finished = Signal()
    error = Signal(str)

    def __init__(
        self,
        parent,
        bin_path: Path,
        ic_path: Path,
        output_path: Path,
        config_path: Path,
        sim_args: list[str],
    ):
        super().__init__(parent)

        self.bin_path = bin_path
        self.ic_path = ic_path
        self.output_path = output_path
        self.config_path = config_path
        self.sim_args = sim_args

        self.sim_process: QProcess | None = None
        self.visualizer: Visualizer | None = None

        self.loading_dialog: QProgressDialog | None = QProgressDialog("Running simulation...", "", 0, 0)
        self.loading_dialog.setWindowTitle("Simulation In Progress")
        self.loading_dialog.setCancelButton(None)
        self.loading_dialog.setWindowModality(Qt.WindowModal) # type: ignore
        self.loading_dialog.setMinimumDuration(0)

    def start(self):
        if self.loading_dialog:
            self.loading_dialog.show()
        self._start_sim_process()

    def _start_sim_process(self):
        if not self.bin_path.is_file():
            self._fail(f"Binary not found: {self.bin_path}")
            return
        
        self.sim_process = QProcess(self)
        self.sim_process.setProgram(str(self.bin_path))
        self.sim_process.setArguments(self.sim_args)
        self.sim_process.setProcessChannelMode(QProcess.MergedChannels)  # type: ignore
        self.sim_process.finished.connect(self._on_sim_process_finished)
        self.sim_process.errorOccurred.connect(self._on_sim_process_error)
        self.sim_process.start()


    def _on_sim_process_finished(self, exit_code, exit_status):
        output = ""
        if self.sim_process:
            output = self.sim_process.readAllStandardOutput().data().decode(errors="ignore")  # type: ignore
            self.sim_process.deleteLater()
            self.sim_process = None
        if exit_status == QProcess.ExitStatus.NormalExit and exit_code == 0:
            self._start_visualizer_load()
        else:
            self._fail(f"Physics Engine Error:\n{output}")


    def _on_sim_process_error(self, error):
        output = ""
        if self.sim_process:
            output = self.sim_process.readAllStandardOutput().data().decode(errors="ignore")  # type: ignore
            self.sim_process.deleteLater()
            self.sim_process = None
        self._fail(output or str(error))

    def _start_visualizer_load(self):
        if self.loading_dialog:
            self.loading_dialog.setLabelText("Loading visualization data...")

        self._loading_thread = QThread(self)
        self._loading_worker = LoadVisualizerWorker(self.output_path,self.config_path)
        self._loading_worker.moveToThread(self._loading_thread)

        self._loading_thread.started.connect(self._loading_worker.run)
        self._loading_worker.finished.connect(self._on_visualizer_loaded)
        self._loading_worker.finished.connect(self._loading_thread.quit)
        self._loading_worker.finished.connect(self._loading_worker.deleteLater)
        self._loading_worker.error.connect(self._fail)
        self._loading_worker.error.connect(self._loading_thread.quit)
        self._loading_worker.error.connect(self._loading_worker.deleteLater)
        self._loading_thread.finished.connect(self._loading_thread.deleteLater)

        self._loading_thread.start()

    def _on_visualizer_loaded(self, data, config):
        self.visualizer = Visualizer(data, config)
        self.visualizer.setAttribute(Qt.WA_DeleteOnClose, True) # type: ignore
        self.visualizer.destroyed.connect(self._on_visualizer_closed)
        self._close_loading_dialog()
        self.visualizer.show()

    def _on_visualizer_closed(self):
        self.visualizer = None
        self.finished.emit()
        self.deleteLater()

    def _close_loading_dialog(self):
        if self.loading_dialog:
            self.loading_dialog.close()
            self.loading_dialog.deleteLater()
            self.loading_dialog = None

    def _fail(self, message: str):
        self._close_loading_dialog()
        self.error.emit(message)
        self.finished.emit() 
        self.deleteLater()

    def cleanup_run_files(self):
        run_dir = self.ic_path.parent
        if not run_dir.exists():
            return
        if not run_dir.name.startswith("run_"):
            print(f"Warning: Refusing to delete directory that doesn't look like a run dir: {run_dir}")
            return
        try:
            shutil.rmtree(run_dir)
            print(f"Cleaned up run directory: {run_dir}")
        except Exception as e:
            print(f"Warning: Failed to clean up {run_dir}: {e}")
 