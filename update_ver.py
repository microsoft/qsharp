import sys
import re
import os

if len(sys.argv) < 2:
    print('Argument is missing. Please specify the new package version, e.g. 0.0.8')
    sys.exit()

newVer = sys.argv[1]
scriptDir = os.path.dirname(os.path.abspath(__file__))

for fileRPath in [
                    f'{scriptDir}/compiler/qsc/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_ast/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_codegen/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_data_structures/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_eval/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_fir/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_frontend/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_hir/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_parse/Cargo.toml',
                    f'{scriptDir}/compiler/qsc_passes/Cargo.toml',
                    f'{scriptDir}/fuzz/Cargo.toml',
                    f'{scriptDir}/katas/Cargo.toml',
                    f'{scriptDir}/language_service/Cargo.toml',
                    f'{scriptDir}/library/tests/Cargo.toml',
                    f'{scriptDir}/pip/Cargo.toml',
                    f'{scriptDir}/pip/pyproject.toml',
                    f'{scriptDir}/wasm/Cargo.toml',

                    f'{scriptDir}/jupyterlab/package.json',
                    f'{scriptDir}/npm/package.json',
                    f'{scriptDir}/playground/package.json',
                    f'{scriptDir}/vscode/package.json'
                ]:
    print(fileRPath)

    # Config:
    regexp = '^version\s*=\s*"\d+\.\d+\.\d+"\s*$'                   # `version = "0.0.11"`
    replacement = f'version = "{newVer}"\n'
    if fileRPath.endswith('package.json'):
        regexp = '\s*"version"\s*:\s*"\d+\.\d+\.\d+"\s*,\s*$'       # `  "version": "0.0.11",`
        replacement = f'  "version": "{newVer}",\n'

    # Read file:
    with open(fileRPath, 'r') as file:
        lines = file.readlines()

    # Replace the line:
    lineIndex = 0           # Zero-based.
    for line in lines:
        if re.match(regexp, line):
            lines[lineIndex] = replacement
            print(f"{lineIndex + 1}: {lines[lineIndex]}", end="")
            break
        lineIndex = lineIndex + 1

    # Save file:
    with open(fileRPath, 'w') as file:
        file.writelines(lines)
