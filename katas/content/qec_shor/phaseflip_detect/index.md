**Input**: three qubits that are either in the state $\ket{\psi_L} = \alpha \ket{+++} + \beta \ket{---}$
or in one of the states $(Z \otimes I \otimes I)\ket{\psi_L}$, $(I \otimes Z \otimes I)\ket{\psi_L}$, or $(I \otimes I \otimes Z)\ket{\psi_L}$ (that is, either in a valid code word of the phase flip code or a code word with a $Z$ error occurring on one of the qubits).

**Goal**: determine whether a $Z$ error has occurred, and if so, on which qubit. 
The return value should be the index of the qubit on which the error occurred, or $-1$ if no error occurred.
The state of the qubits after your operation is applied shouldn't change.
