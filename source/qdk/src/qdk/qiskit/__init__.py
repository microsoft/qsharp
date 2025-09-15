try:
    import qiskit
except ImportError:
    raise ImportError(
        'qdk.qiskit requires the qiskit package. Please install it via: pip install "qdk[qiskit]"'
    )

from qsharp.interop.qiskit import *
from qsharp import TargetProfile
