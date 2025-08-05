# TODO

- Compare to Qiskit Aer APIs/model and align where possible.
- Look to integrate with Qiskit Aer simulator
- Test perf compared to Qiskit Aer on CPU/GPU
  - Note: For qiskit-aer-gpu, requires Python 3.12 and Linux x64 (20MB download)
  - Looks like difference between latest CPU (17.1) and latest GPU 15.1 is minimal
  - Lots of good sample tests under tests/terra
- Refactor how Ops stores qubits and other args
- Add noise on measurements and initialization
- Test and clean up JSON noise file format
- Test and clean up the noise utility functions
- Add ccx, reset, and mresetz instructions to default set
- Try to run on/compare with with CUDA-Q
