from PySide6.QtWidgets import QApplication
from launcher import Launcher


def main():
    app = QApplication()
    launcher = Launcher()
    launcher.show()
    return app.exec()

if __name__ == '__main__':
    main()
