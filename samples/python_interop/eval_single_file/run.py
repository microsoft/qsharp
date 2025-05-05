from pathlib import Path
import qsharp

# Import the Q# code from the teleport.qs file
code = (Path(__file__).parent / "sample.qs").read_text()
qsharp.eval(code)

# Directly invoke the Main operation defined in the loaded file
print(qsharp.code.Main())
