# Q# Python Bindings

## TODO:
- [ ] Fix state visualization on the JS side to match
- [ ] Finalize error/exception types. Base class? Single type?
- [ ] Fancier error reporting through miette
- [ ] See if runtime errors have stack traces.
- [ ] Clean up Rust code
- [ ] Write README.md (usage examples)
- [ ] Set proper version in pip/qsharp/Cargo.toml and pyproject.toml
- [ ] Can we share Python version between environment.yml and build.py somehow?
- [ ] Clean up __IPYTHON__ initialization in __init__.py (global registration? Pylance squigglies?)
- [ ] Finalize Value types to marshal 
- [ ] Update _native.pyi with implementation (can this be generated?? https://mypy.readthedocs.io/en/stable/stubgen.html or make-stub-files)
- [ ] Refine public api in __init__.py
- [ ] Clean up ipython.py
- [ ] Clean up qsharp.py
- [ ] Convert samples.py to unit tests
- [ ] Rename test file (test_interpreter.py?)
- [ ] Check in sample notebook and sample.qs
- [ ] Figure out how to open namespaces
- [ ] Eliminate pip/qsharp subdirectory?
- [ ] Panic handler
- [ ] Clean up TODOs across the board
