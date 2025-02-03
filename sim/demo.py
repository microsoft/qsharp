from pathlib import Path
from disco.wasm_runner import qir_to_wasm, run_wasm

# Update this file for the script to run
qir_script = "main.c"

if __name__ == "__main__":
    qir_file_path = Path(__file__).parent / qir_script
    wasm_file_path = qir_to_wasm(qir_file_path)

    with open(wasm_file_path, "rb") as f:
        wasm_module = f.read()

    shot_result = run_wasm(wasm_module, 25)
    for result in shot_result:
        print("".join(result))
