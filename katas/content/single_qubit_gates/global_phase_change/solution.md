We recognize that a global phase change can be accomplished by using the R rotation gate with the PauliI (identity) gate.
As a reminder, the R gate is defined as $R_{\mu}(\theta) = \exp(\frac{\theta}{2}i\cdot\sigma_{\mu})$, where $\sigma_{\mu}$ is one of the Pauli gates I, X, Y or Z.

> Note that a global phase is not detectable and has no physical meaning - it disappears when you take a measurement of the state.  
> You can read more about this in the [Single-qubit measurements tutorial](../tutorials/SingleQubitSystemMeasurements/SingleQubitSystemMeasurements.ipynb#Measurements-in-arbitrary-orthogonal-bases).

For the problem at hand, we'll use the rotation gate $R_{\mu}(\theta) = \exp(\frac{\theta}{2}i\cdot\sigma_{\mu})$ with $\sigma_{\mu} = I$.

$$R(PauliI, 2\pi) = \exp(\frac{2\pi}{2} iI) = \exp(i\pi) I = (\cos\pi + i\sin\pi) I$$
Since $\cos\pi = -1$ and $\sin\pi = 0$, we have that $R(PauliI, 2\pi) = -I$:

$$
R(\beta |0\rangle + \gamma |1\rangle) =
 -1\begin{bmatrix}1 & 0 \\\ 0 & 1 \end{bmatrix}
 \begin{bmatrix}\beta\\\ \gamma\\\ \end{bmatrix}=
 \begin{bmatrix}-1 & 0 \\\ 0 & -1 \end{bmatrix}
 \begin{bmatrix}\beta\\\ \gamma\\\ \end{bmatrix} =
 \begin{bmatrix}(-1)\cdot\beta + 0\cdot\gamma\\\ 0\cdot\beta +  (-1)\cdot\gamma \\\ \end{bmatrix}=
\begin{bmatrix}-\beta\\\ -\gamma\\\ \end{bmatrix}=
-\beta |0\rangle - \gamma |1\rangle
$$

The test harness for this test shows the result of applying the _controlled_ variant of your solution to be able to detect the phase change.
@[solution]({
"id": "single_qubit_gates__global_phase_change_solution",
"codePath": "./Solution.qs"
})
