# ApplyCNOTChain operation

`operation ApplyCNOTChain(qubits : Qubit[]) : Unit is Adj + Ctl`

## Summary
Computes the parity of a register of qubits in-place.

## Input
### qubits
Array of qubits whose parity is to be computed and stored.

## Remarks
This operation transforms the state of its input asd
$$
\begin{align}
    \ket{q_0} \ket{q_1} \cdots \ket{q_{n - 1}} & \mapsto
    \ket{q_0} \ket{q_0 \oplus q_1} \ket{q_0 \oplus q_1 \oplus q_2} \cdots
        \ket{q_0 \oplus \cdots \oplus q_{n - 1}}.
\end{align}
$$
