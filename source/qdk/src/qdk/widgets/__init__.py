try:
    from qsharp_widgets import *
except ImportError:
    raise ImportError(
        'qdk.widgets requires the qsharp_widgets package. Please install it via: pip install "qdk[widgets]"'
    )
