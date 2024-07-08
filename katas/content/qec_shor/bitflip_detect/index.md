**Input**: three qubits that are either in the state $\ket{\psi_L} = \alpha \ket{000} + \beta \ket{111}$
or in one of the states $(X \otimes I \otimes I)\ket{\psi_L}$, $(I \otimes X \otimes I)\ket{\psi_L}$, or $(I \otimes I \otimes X)\ket{\psi_L}$ (that is, either in a valid code word of the bit flip code or a code word with an $X$ error occurring on one of the qubits).

**Goal**: determine whether an $X$ error has occurred, and if so, on which qubit. 
The return value should be the index of the qubit on which the error occurred, or $-1$ if no error occurred.
The state of the qubits after your operation is applied shouldn't change.
