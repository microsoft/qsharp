# operation CCNOT(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj + Ctl

## Summary
Applies the doubly controlled–NOT (CCNOT) gate to three qubits.

## Input
### control1
First control qubit for the CCNOT gate.
### control2
Second control qubit for the CCNOT gate.
### target
Target qubit for the CCNOT gate.

## Remarks
Equivalent to:
```qsharp
Controlled X([control1, control2], target);
```

&nbsp;

---

&nbsp;

# operation CNOT(control : Qubit, target : Qubit) : Unit is Adj + Ctl

## Summary
Applies the controlled-NOT (CNOT) gate to a pair of qubits.

## Input
### control
Control qubit for the CNOT gate.
### target
Target qubit for the CNOT gate.

## Remarks
$$
\begin{align}
    \operatorname{CNOT} \mathrel{:=}
    \begin{bmatrix}
        1 & 0 & 0 & 0 \\\\
        0 & 1 & 0 & 0 \\\\
        0 & 0 & 0 & 1 \\\\
        0 & 0 & 1 & 0
    \end{bmatrix},
\end{align}
$$

where rows and columns are ordered as in the quantum concepts guide.

Equivalent to:
```qsharp
Controlled X([control], target);
```

&nbsp;

---

&nbsp;

# operation Exp(paulis : Pauli[], theta : Double, qubits : Qubit[]) : Unit is Adj + Ctl

## Summary
Applies the exponential of a multi-qubit Pauli operator.

## Input
### paulis
Array of single-qubit Pauli values indicating the tensor product
factors on each qubit.
### theta
Angle about the given multi-qubit Pauli operator by which the
target register is to be rotated.
### qubits
Register to apply the given rotation to.

## Remarks
$$
\begin{align}
    e^{i \theta [P_0 \otimes P_1 \cdots P_{N-1}]},
\end{align}
$$
where $P_i$ is the $i$th element of `paulis`, and where
$N = $`Length(paulis)`.

&nbsp;

---

&nbsp;

# operation H(qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies the Hadamard transformation to a single qubit.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    e^{i \theta [P_0 \otimes P_1 \cdots P_{N-1}]},
\end{align}
$$
where $P_i$ is the $i$th element of `paulis`, and where
$N = $`Length(paulis)`.

&nbsp;

---

&nbsp;

# operation I(target : Qubit) : Unit is Adj + Ctl

## Summary
Performs the identity operation (no-op) on a single qubit.

## Remarks
This is a no-op. It is provided for completeness and because
sometimes it is useful to call the identity in an algorithm or to pass it as a parameter.

&nbsp;

---

&nbsp;

# operation M(qubit : Qubit) : Result

## Summary
Performs a measurement of a single qubit in the
Pauli _Z_ basis.

## Input
### qubit
Qubit to be measured.

## Output
`Zero` if the +1 eigenvalue is observed, and `One` if
the -1 eigenvalue is observed.

## Remarks
The output result is given by
the distribution
$$
\begin{align}
    \Pr(\texttt{Zero} | \ket{\psi}) =
        \braket{\psi | 0} \braket{0 | \psi}.
\end{align}
$$

Equivalent to:
```qsharp
Measure([PauliZ], [qubit]);
```

&nbsp;

---

&nbsp;

# operation Measure(bases : Pauli[], qubits : Qubit[]) : Result

## Summary
Performs a joint measurement of one or more qubits in the
specified Pauli bases.

## Input
### bases
Array of single-qubit Pauli values indicating the tensor product
factors on each qubit.
### qubits
Register of qubits to be measured.

## Output
`Zero` if the +1 eigenvalue is observed, and `One` if
the -1 eigenvalue is observed.

## Remarks
The output result is given by the distribution:
$$
\begin{align}
    \Pr(\texttt{Zero} | \ket{\psi}) =
        \frac12 \braket{
            \psi \mid|
            \left(
                \boldone + P_0 \otimes P_1 \otimes \cdots \otimes P_{N-1}
            \right) \mid|
            \psi
        },
\end{align}
$$
where $P_i$ is the $i$th element of `bases`, and where
$N = \texttt{Length}(\texttt{bases})$.
That is, measurement returns a `Result` $d$ such that the eigenvalue of the
observed measurement effect is $(-1)^d$.

If the basis array and qubit array are different lengths, then the
operation will fail.

&nbsp;

---

&nbsp;

# operation R(pauli : Pauli, theta : Double, qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies a rotation about the given Pauli axis.

## Input
### pauli
Pauli operator (μ) to be exponentiated to form the rotation.
### theta
Angle in radians about which the qubit is to be rotated.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_{\mu}(\theta) \mathrel{:=}
    e^{-i \theta \sigma_{\mu} / 2},
\end{align}
$$
where $\mu \in \{I, X, Y, Z\}$.

When called with `pauli = PauliI`, this operation applies
a *global phase*. This phase can be significant
when used with the `Controlled` functor.

&nbsp;

---

&nbsp;

# operation R1(theta : Double, qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies a rotation about the |1⟩ state by a given angle.

## Input
### theta
Angle about which the qubit is to be rotated.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_1(\theta) \mathrel{:=}
    \operatorname{diag}(1, e^{i\theta}).
\end{align}
$$

Equivalent to:
```qsharp
R(PauliZ, theta, qubit);
R(PauliI, -theta, qubit);
```

&nbsp;

---

&nbsp;

# operation R1Frac(numerator : Int, power : Int, qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies a rotation about the |1⟩ state by an angle specified
as a dyadic fraction.

WARNING:
This operation uses the **opposite** sign convention from
Microsoft.Quantum.Intrinsic.R, and does not include the
factor of 1/2 included by Microsoft.Quantum.Intrinsic.R1.

## Input
### numerator
Numerator in the dyadic fraction representation of the angle
by which the qubit is to be rotated. This angle is expressed in radians.
### power
Power of two specifying the denominator of the angle by which
the qubit is to be rotated. This angle is expressed in radians.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_1(n, k) \mathrel{:=}
    \operatorname{diag}(1, e^{i \pi k / 2^n}).
\end{align}
$$

Equivalent to:
```qsharp
RFrac(PauliZ, -numerator, denominator + 1, qubit);
RFrac(PauliI, numerator, denominator + 1, qubit);
```

&nbsp;

---

&nbsp;

# operation Reset(qubit : Qubit) : Unit

## Summary
Given a single qubit, measures it and ensures it is in the |0⟩ state
such that it can be safely released.

## Input
### qubit
The qubit whose state is to be reset to |0⟩.

&nbsp;

---

&nbsp;

# operation ResetAll(qubits : Qubit[]) : Unit

## Summary
Given an array of qubits, measure them and ensure they are in the |0⟩ state
such that they can be safely released.

## Input
### qubits
An array of qubits whose states are to be reset to |0⟩.

&nbsp;

---

&nbsp;

# operation RFrac(pauli : Pauli, numerator : Int, power : Int, qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies a rotation about the given Pauli axis by an angle specified
as a dyadic fraction.

WARNING:
This operation uses the **opposite** sign convention from
Microsoft.Quantum.Intrinsic.R.

## Input
### pauli
Pauli operator to be exponentiated to form the rotation.
### numerator
Numerator in the dyadic fraction representation of the angle
by which the qubit is to be rotated. This angle is expressed in radians.
### power
Power of two specifying the denominator of the angle by which
the qubit is to be rotated. This angle is expressed in radians.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_{\mu}(n, k) \mathrel{:=}
    e^{i \pi n \sigma_{\mu} / 2^k},
\end{align}
$$
where $\mu \in \{I, X, Y, Z\}$.

Equivalent to:
```qsharp
// PI() is a Q# function that returns an approximation of π.
R(pauli, -PI() * IntAsDouble(numerator) / IntAsDouble(2 ^ (power - 1)), qubit);
```

&nbsp;

---

&nbsp;

# operation Rx(theta : Double, qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies a rotation about the _x_-axis by a given angle.

## Input
### theta
Angle about which the qubit is to be rotated.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_x(\theta) \mathrel{:=}
    e^{-i \theta \sigma_x / 2} =
    \begin{bmatrix}
        \cos \frac{\theta}{2} & -i\sin \frac{\theta}{2}  \\\\
        -i\sin \frac{\theta}{2} & \cos \frac{\theta}{2}
    \end{bmatrix}.
\end{align}
$$

Equivalent to:
```qsharp
R(PauliX, theta, qubit);
```

&nbsp;

---

&nbsp;

# operation Rxx(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl

## Summary
Applies the two qubit Ising _XX_ rotation gate.

## Input
### theta
The angle about which the qubits are rotated.
### qubit0
The first qubit input to the gate.
### qubit1
The second qubit input to the gate.

## Remarks
$$
\begin{align}
    R_{xx}(\theta) \mathrel{:=}
    \begin{bmatrix}
        \cos \theta & 0 & 0 & -i\sin \theta  \\\\
        0 & \cos \theta & -i\sin \theta & 0  \\\\
        0 & -i\sin \theta & \cos \theta & 0  \\\\
        -i\sin \theta & 0 & 0 & \cos \theta
    \end{bmatrix}.
\end{align}
$$

&nbsp;

---

&nbsp;

# operation Ry(theta : Double, qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies a rotation about the _y_-axis by a given angle.

## Input
### theta
Angle about which the qubit is to be rotated.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_y(\theta) \mathrel{:=}
    e^{-i \theta \sigma_y / 2} =
    \begin{bmatrix}
        \cos \frac{\theta}{2} & -\sin \frac{\theta}{2}  \\\\
        \sin \frac{\theta}{2} & \cos \frac{\theta}{2}
    \end{bmatrix}.
\end{align}
$$

Equivalent to:
```qsharp
R(PauliY, theta, qubit);
```

&nbsp;

---

&nbsp;

# operation Ryy(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl

## Summary
Applies the two qubit Ising _YY_ rotation gate.

## Input
### theta
The angle about which the qubits are rotated.
### qubit0
The first qubit input to the gate.
### qubit1
The second qubit input to the gate.

## Remarks
$$
\begin{align}
    R_{yy}(\theta) \mathrel{:=}
    \begin{bmatrix}
        \cos \theta & 0 & 0 & i\sin \theta  \\\\
        0 & \cos \theta & -i\sin \theta & 0  \\\\
        0 & -i\sin \theta & \cos \theta & 0  \\\\
        i\sin \theta & 0 & 0 & \cos \theta
    \end{bmatrix}.
\end{align}
$$

&nbsp;

---

&nbsp;

# operation Rz(theta : Double, qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies a rotation about the _z_-axis by a given angle.

## Input
### theta
Angle about which the qubit is to be rotated.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_z(\theta) \mathrel{:=}
    e^{-i \theta \sigma_z / 2} =
    \begin{bmatrix}
        e^{-i \theta / 2} & 0 \\\\
        0 & e^{i \theta / 2}
    \end{bmatrix}.
\end{align}
$$

Equivalent to:
```qsharp
R(PauliZ, theta, qubit);
```

&nbsp;

---

&nbsp;

# operation Rzz(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl

## Summary
Applies the two qubit Ising _ZZ_ rotation gate.

## Input
### theta
The angle about which the qubits are rotated.
### qubit0
The first qubit input to the gate.
### qubit1
The second qubit input to the gate.

## Remarks
$$
\begin{align}
    R_{zz}(\theta) \mathrel{:=}
    \begin{bmatrix}
        e^{-i \theta / 2} & 0 & 0 & 0 \\\\
        0 & e^{i \theta / 2} & 0 & 0 \\\\
        0 & 0 & e^{i \theta / 2} & 0 \\\\
        0 & 0 & 0 & e^{-i \theta / 2}
    \end{bmatrix}.
\end{align}
$$

&nbsp;

---

&nbsp;

# operation S(qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies the π/4 phase gate to a single qubit.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    S \mathrel{:=}
    \begin{bmatrix}
        1 & 0 \\\\
        0 & i
    \end{bmatrix}.
\end{align}
$$

&nbsp;

---

&nbsp;

# operation SWAP(qubit1 : Qubit, qubit2 : Qubit) : Unit is Adj + Ctl

## Summary
Applies the SWAP gate to a pair of qubits.

## Input
### qubit1
First qubit to be swapped.
### qubit2
Second qubit to be swapped.

## Remarks
$$
\begin{align}
    \operatorname{SWAP} \mathrel{:=}
    \begin{bmatrix}
        1 & 0 & 0 & 0 \\\\
        0 & 0 & 1 & 0 \\\\
        0 & 1 & 0 & 0 \\\\
        0 & 0 & 0 & 1
    \end{bmatrix},
\end{align}
$$

where rows and columns are ordered as in the quantum concepts guide.

Equivalent to:
```qsharp
CNOT(qubit1, qubit2);
CNOT(qubit2, qubit1);
CNOT(qubit1, qubit2);
```

&nbsp;

---

&nbsp;

# operation T(qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies the π/8 gate to a single qubit.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    T \mathrel{:=}
    \begin{bmatrix}
        1 & 0 \\\\
        0 & e^{i \pi / 4}
    \end{bmatrix}.
\end{align}
$$

&nbsp;

---

&nbsp;

# operation X(qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies the Pauli _X_ gate.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    \sigma_x \mathrel{:=}
    \begin{bmatrix}
        0 & 1 \\\\
        1 & 0
    \end{bmatrix}.
\end{align}
$$

&nbsp;

---

&nbsp;

# operation Y(qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies the Pauli _Y_ gate.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    \sigma_y \mathrel{:=}
    \begin{bmatrix}
        0 & -i \\\\
        i & 0
    \end{bmatrix}.
\end{align}
$$

&nbsp;

---

&nbsp;

# operation Z(qubit : Qubit) : Unit is Adj + Ctl

## Summary
Applies the Pauli _Z_ gate.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    \sigma_z \mathrel{:=}
    \begin{bmatrix}
        1 & 0 \\\\
        0 & -1
    \end{bmatrix}.
\end{align}
$$

&nbsp;

---

&nbsp;

# function Message(msg : String) : Unit

## Summary
Logs a message.

## Input
### msg
The message to be reported.

## Remarks
The specific behavior of this function is simulator-dependent,
but in most cases the given message will be written to the console.
```
