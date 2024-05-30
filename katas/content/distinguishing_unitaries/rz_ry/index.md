**Inputs:** 

1. An angle $\theta \in [0.01 \pi; 0.99 \pi]$.
2. An operation that implements a single-qubit unitary transformation:
either the [$R_z(\theta)$ gate](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.intrinsic/rz) or the [$R_y(\theta)$ gate](https://learn.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic/ry). 

**Output:**  0 if the given operation is the $R_z$ gate, 1 if the given operation is the $R_y$ gate.

The operation will have Adjoint and Controlled variants defined.
You are allowed to apply the given operation and its adjoint/controlled variants **any number of times**.