We start with the state $|\psi_0\rangle=|00\rangle$.

**Step 1:** Start by putting the first qubit in the state $\alpha|0\rangle + \beta|1\rangle$, where $\alpha$ and $\beta$ are the square roots of relative weights of all basis states which start with 0 and with 1, respectively.

In case of Hardy state, two basis states start with 0: $\frac{1}{\sqrt{12}} \big (3|00\rangle + |01\rangle\big )$ and two basis states start with 1: $\frac{1}{\sqrt{12}} \big (|10\rangle + |11\rangle\big )$.
The relative weights of 0 and 1 are just the sums of squares of their amplitudes:
$\alpha^2 = \frac{9}{12} + \frac{1}{12} = \frac{10}{12}$ and $\beta^2 = \frac{1}{12} + \frac{1}{12} = \frac{2}{12}$, respectively.
So we'll need to put the first qubit in the state $\sqrt{\frac{10}{12}}|0\rangle + \sqrt{\frac{2}{12}}|1\rangle$ using the $R_y$ gate:

$$|00\rangle \overset{R_{y_1}}\rightarrow \big (\sqrt{\frac{10}{12}}|0\rangle + \sqrt{\frac{2}{12}}|1\rangle \big ) \otimes |0\rangle =: |\psi_1\rangle$$

Here $R_{y_1} := R_y(2\arccos \sqrt{\frac{10}{12}}) \otimes I$.

@[solution]({
    "id": "superposition__hardy_state_solution",
    "codePath": "./Solution.qs"
})
