import sys
from pathlib import Path

from PySide6.QtCore import QObject, QProcess, Qt, Signal
from PySide6.QtWidgets import QProgressDialog
from schema import SimulationParameters


# Base runner class to manage a subprocess
class Runner(QObject):
    finished = Signal(int)  # exit code
    error = Signal(str)  # error message

    def __init__(
        self, program: str, args: list[str], dialog_title: str, dialog_label: str
    ):
        super().__init__()
        self.program = program
        self.args = args
        self.loading_dialog = self.create_loading_dialog(dialog_title, dialog_label)
        self.loading_dialog.canceled.connect(self.cancel)
        self.process = QProcess(self)
        self.process.setProcessChannelMode(QProcess.MergedChannels)  # type: ignore
        self.process.finished.connect(self.on_process_finished)
        self.process.errorOccurred.connect(self.on_process_error)
        self.process.readyReadStandardOutput.connect(self.on_process_output)
        self.cancelled = False

    def start(self):
        self.process.start(self.program, self.args)
        print(f"Started process: {self.program} {' '.join(self.args)}")
        if self.loading_dialog:
            self.loading_dialog.show()

    def on_process_finished(self, exit_code: int, exit_status: QProcess.ExitStatus):
        self.close_loading_dialog()
        if self.process:
            self.process.deleteLater()
        if not self.cancelled and (
            exit_code != 0 or exit_status != QProcess.ExitStatus.NormalExit
        ):
            self.error.emit(f"Process exited (code={exit_code}, status={exit_status})")
        self.finished.emit(exit_code)

    def on_process_error(self, error: QProcess.ProcessError):
        self.close_loading_dialog()
        exit_code = self.process.exitCode() if self.process else -1
        if self.process:
            self.process.deleteLater()
        if not self.cancelled:
            self.error.emit(f"Process error occurred: {str(error)}")
        self.finished.emit(exit_code)

    def create_loading_dialog(self, title: str, label: str) -> QProgressDialog:
        dialog = QProgressDialog(label, "Cancel", 0, 0)  # type: ignore
        dialog.setWindowTitle(title)
        dialog.setWindowModality(Qt.WindowModal)  # type: ignore
        return dialog

    def close_loading_dialog(self) -> None:
        if self.loading_dialog:
            try:
                self.loading_dialog.canceled.disconnect(self.cancel)
            except TypeError:
                pass
            self.loading_dialog.close()
            self.loading_dialog.deleteLater()
            self.loading_dialog = None

    def cancel(self) -> None:
        if self.process and self.process.state() != QProcess.NotRunning:  # type: ignore
            self.cancelled = True
            self.process.terminate()
            self.process.waitForFinished()
            self.close_loading_dialog()

    def on_process_output(self) -> None:
        if self.process:
            output = self.process.readAllStandardOutput().data().decode()  # type: ignore
            print(f"Process output: {output}")


# Runner for the simulation subprocess
# Uses the n-body-sim executable to run the simulation with the given parameters and initial conditions
class SimulationRunner(Runner):
    def __init__(
        self, ic_path: Path, output_path: Path, sim_parameters: SimulationParameters
    ) -> None:
        program = str(
            Path(__file__).parents[1]
            / "bin"
            / ("n-body-sim.exe" if sys.platform == "win32" else "n-body-sim")
        )
        args = [
            "-i",
            str(ic_path),
            "-o",
            str(output_path),
            "-g",
            str(sim_parameters.g_constant),
            "-t",
            str(sim_parameters.time_step),
            "-n",
            str(sim_parameters.num_steps),
            "--softening-factor",
            str(sim_parameters.softening_factor),
            "--theta",
            str(sim_parameters.theta),
            "--gravity",
            sim_parameters.gravity,
            "--integrator",
            sim_parameters.integrator,
        ]
        super().__init__(
            program=program,
            args=args,
            dialog_title="Running Simulation",
            dialog_label="Simulation running. This may take a while...",
        )

    def on_process_output(self) -> None:
        pass  # TODO implement progress output


# Note: Launching visualizer.py as a subprocess rather than importing and running the Visualizer class directly
# because this allows the visualizer's potentially long-running and blocking I/O when loading simulation data
# to be terminated safely and instantly by killing the subprocess. The visualizer loading logic was originally
# implemented to run in a QThread that signalled when loading was complete, which kept the main menu responsive,
# but did not allow graceful early cancellation because threads need to be terminated cooperatively, and the
# visualizer can't check for stop signals while it's blocked on I/O. This approach also matches how the simulation
# binary is launched, simplifying the code for both simulation and visualization.


# Runner for the visualizer subprocess
# Uses the visualizer.py script to visualize the simulation results from the output file
class VisualizerRunner(Runner):
    def __init__(self, output_path: Path, config_path: Path) -> None:
        program = sys.executable
        script_path = Path(__file__).parent / "visualizer.py"
        args = [str(script_path), str(output_path), str(config_path)]
        super().__init__(
            program=program,
            args=args,
            dialog_title="Loading Visualizer",
            dialog_label="Loading simulation data...",
        )

    def on_process_output(self) -> None:
        if self.process:
            output = self.process.readAllStandardOutput().data().decode()  # type: ignore
            for line in output.splitlines():
                if line.strip() == "LOADED":
                    print("Visualizer loaded successfully.")
                    self.close_loading_dialog()
