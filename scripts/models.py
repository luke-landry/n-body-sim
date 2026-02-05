from PySide6.QtCore import Qt, QAbstractTableModel, QModelIndex
from data import BodyConfig

# table model for body configurations in the launcher's table view
class BodyTableModel(QAbstractTableModel):
    def __init__(self, bodies: list[BodyConfig] | None = None):
        super().__init__()
        self.bodies = bodies or [BodyConfig.default(1)]
        self.keys = list(BodyConfig.model_fields.keys())
        self.headers = [k.replace('_', ' ').title() for k in self.keys]

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
            return getattr(body, key)
        return None

    def setData(self, index, value, role=Qt.ItemDataRole.EditRole):
        if not index.isValid():
            return False
        if role != Qt.ItemDataRole.EditRole:
            return False
        
        key = self.keys[index.column()]
        try:
            if key not in ["name", "color"]:
                value = float(value)
            setattr(self.bodies[index.row()], key, value)
            self.dataChanged.emit(index, index)
            return True
        except ValueError:
            return False

    def headerData(self, section, orientation, role):
        if role != Qt.ItemDataRole.DisplayRole:
            return None
        if orientation == Qt.Orientation.Horizontal:
            return self.headers[section]
        if orientation == Qt.Orientation.Vertical:
            return str(section + 1)
        return None

    def flags(self, index):
        return Qt.ItemFlag.ItemIsEditable | Qt.ItemFlag.ItemIsEnabled | Qt.ItemFlag.ItemIsSelectable

    def add_body(self):
        self.beginInsertRows(QModelIndex(), self.rowCount(), self.rowCount())
        self.bodies.append(BodyConfig.default(len(self.bodies) + 1))
        self.endInsertRows()

    def remove_body(self, row):
        if 0 <= row < len(self.bodies):
            self.beginRemoveRows(QModelIndex(), row, row)
            self.bodies.pop(row)
            self.endRemoveRows()
            return True
        return False
