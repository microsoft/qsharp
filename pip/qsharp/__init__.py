# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import init, eval, eval_file, run, compile, dump_machine

from ._native import Result, Pauli, QSharpError, TargetProfile, StateDump

# IPython notebook specific features
try:
    if __IPYTHON__:  # type: ignore
        from ._ipython import register_magic, enable_classic_notebook_codemirror_mode

        register_magic()
        enable_classic_notebook_codemirror_mode()
except NameError:
    pass


__all__ = [
    "init",
    "eval",
    "eval_file",
    "run",
    "dump_machine",
    "compile",
    "Result",
    "Pauli",
    "QSharpError",
    "TargetProfile",
    "StateDump"
]

def qsc():
    # Get file paths to compile from command line arguments
    import os
    import argparse
    from ._native import Interpreter, TargetProfile, PackageType
    parser = argparse.ArgumentParser(description="Q# compiler")
    parser.add_argument("files", nargs="*", help="Q# source files to compile.")
    parser.add_argument("--target", default="Full", choices=["Full", "Base"], help="The target profile to use for compilation.")
    parser.add_argument("--nostdlib", action='store_true', help="Skip including the standard library.")
    args = parser.parse_args()
    file_paths = args.files
    target_profile = getattr(TargetProfile, args.target)
    # Get the contents of the files
    sources = []
    for file_path in file_paths:
        with open(file_path, mode="r", encoding="utf-8") as f:
            sources.append((os.path.basename(file_path), f.read()))
    # Compile the files
    try:
        _ = Interpreter(target_profile, PackageType.Lib, args.nostdlib, sources)
    except QSharpError as e:
        print(e)
        exit(1)

def qsi():
    # Get file paths to compile from command line arguments
    import sys
    import os
    import argparse
    from ._native import Interpreter, TargetProfile, PackageType
    parser = argparse.ArgumentParser(description="Q# interpreter")
    parser.add_argument("files", nargs="*", help="Q# source files to interpret.")
    parser.add_argument("--target", default="Full", choices=["Full", "Base"], help="The target profile to use for compilation.")
    parser.add_argument("--entry", default=None, help="An optional entry expression to interpret. If none is provided, the source files must include an entry point.")
    parser.add_argument("--shots", default=1, help="The number of shots to run.")
    parser.add_argument("--exec", action='store_true', help="Exit after loading the files or running the given file(s)/entry on the command line.")
    parser.add_argument("--nostdlib", action='store_true', help="Skip including the standard library.")
    args = parser.parse_args()
    file_paths = args.files
    target_profile = getattr(TargetProfile, args.target)
    # Get the contents of the files
    sources = []
    for file_path in file_paths:
        with open(file_path, mode="r", encoding="utf-8") as f:
            sources.append((os.path.basename(file_path), f.read()))
    # Compile the files and run the specified number of shots
    try:
        package_type = PackageType.Exe if args.entry is None and args.exec else PackageType.Lib
        interpreter = Interpreter(target_profile, package_type, args.nostdlib, sources)
        if args.exec:
            for res in interpreter.run(int(args.shots), lambda output: print(output), args.entry):
                print(res)
            exit(0)
        else:
            # Use readline to enable history and arrow key support
            import readline
            if args.entry is not None:
                interpreter.interpret(args.entry, lambda output: print(output))
            while True:
                try:
                    snippet = input("qsi$ ")
                    res = interpreter.interpret(snippet, lambda output: print(output))
                    if res is not None:
                        print(res)
                except QSharpError as e:
                    print(e)
                except KeyboardInterrupt:
                    exit(0)
    except QSharpError as e:
        print(e)
        exit(1)
