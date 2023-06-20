## Measurements and entanglement

Qubits entanglement has an effect on the measurement statistics of the system. If two qubits are entangled, then their measurement outcomes will be correlated, while separable states (which are by definition not entangled) have uncorrelated measurement outcomes.

> It is useful to revisit the concepts of entanglement and separable states, which were introduced in the [tutorial on multi-qubit systems](../MultiQubitSystems/MultiQubitSystems.ipynb#Entanglement). Consider a system of $n>1$ number of qubits, which we divide into two parts: A, consisting of $m$ qubits, and B, consisting of the remaining $n-m$ qubits. We say that the state $\ket \psi$ of the entire system is separable if it can be expressed as a tensor product of the states of parts A and B: 
$$
\ket \psi = \ket {\phi_A} \otimes \ket{\phi_B}
$$
where $\ket{\phi_A}$ and $\ket{\phi_B}$ are wave functions that describe parts $A$ and $B$, respectively. If it is not possible to express $\ket \psi$ in such a form, then we say that system A is entangled with system B.

Consider a measurement on the subsystem $A$ of a separable state. Let the measurement be done in a basis $\{ \ket{b_0},\dotsc,\ket{b_{2^m-1}}\}$. According to the projection formalism, a projection operator $P_i = \ket{b_i}\bra{b_i} \otimes \mathbb{1}$ is chosen randomly. The corresponding post-measurement state of the system is then given by
\begin{align*}
\ket{\psi}_{i} &\equiv \frac{P_i \ket{\psi}}{\big|P_i \ket{\psi}\big|} = \\
               &= \frac{\ket{b_i}\bra{b_i}\phi_A\rangle \otimes \ket {\phi_B}}{\big|\ket{b_i}\bra{b_i}\phi_A\rangle \otimes \ket {\phi_B}\big|} = \\
               &= \frac{\bra{b_i}\phi_A\rangle \cdot \ket{b_i} \otimes \ket {\phi_B}}{\big|\ket{b_i}\big| \cdot \bra{b_i}\phi_A\rangle \cdot \big| \ket {\phi_B}\big|} = \\
               &= \ket{b_i} \otimes \ket{\phi_B}.
\end{align*}

Thus, the state of subsystem $B$ after the measurement is $\ket{\phi_B}$ independently of the outcome $i$ of the measurement on the first qubit. The results of a subsequent measurement on subsystem $B$, including outcome probabilities, will be independent of the result of the first measurement. In other words, the outcomes of the two measurements will be uncorrelated.

On the other hand, if the system is entangled, then the measurement outcomes will be correlated, in a manner dictated by the bases chosen for the measurements on the two subsystems. The following exercise illustrates this phenomenon.

### <span style="color:blue">Exercise 6</span>: Sequential measurements on an entangled state and a separable state
Consider two two-qubit states:
- The Bell state $|\Phi^{+}\rangle = \frac{1}{\sqrt{2}} \big (|00\rangle + |11\rangle\big)$.
- A state $\ket \Theta = \frac{1}{2} \big( \ket{00} + \ket{01} + \ket{10} + \ket{11} \big)$.

For both states, consider a measurement on the first qubit, followed by a measurement on the second qubit, both done in the computational basis. For which state can we expect the measurement outcomes to be correlated? Verify by calculating the sequential measurement probabilities explicitly for both states. 

*Can't come up with a solution? See the explained solution in the [Multi-Qubit System Measurement Workbook](./Workbook_MultiQubitSystemMeasurements.ipynb#Exercise-6:-Sequential-measurements-on-an-entangled-state-and-a-separable-state).*

> Congratulations! You have learned enough about the measurements to start solving problems from the [Measurements kata](../../Measurements/Measurements.ipynb)!
You can give them a try and return to the rest of this tutorial later, or keep reading.
