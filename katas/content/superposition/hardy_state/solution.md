We start with the state $|\psi_0\rangle=|00\rangle$.

**Step 1:** Start by putting the first qubit in the state $\alpha|0\rangle + \beta|1\rangle$, where $\alpha$ and $\beta$ are the square roots of relative weights of all basis states which start with 0 and with 1, respectively.

In case of Hardy state, two basis states start with 0: $\frac{1}{\sqrt{12}} \big (3|00\rangle + |01\rangle\big )$ and two basis states start with 1: $\frac{1}{\sqrt{12}} \big (|10\rangle + |11\rangle\big )$.
The relative weights of 0 and 1 are just the sums of squares of their amplitudes:
$\alpha^2 = \frac{9}{12} + \frac{1}{12} = \frac{10}{12}$ and $\beta^2 = \frac{1}{12} + \frac{1}{12} = \frac{2}{12}$, respectively.
So we'll need to put the first qubit in the state $\sqrt{\frac{10}{12}}|0\rangle + \sqrt{\frac{2}{12}}|1\rangle$ using the $R_y$ gate:

$$|00\rangle \overset{R_{y_1}}\rightarrow \big (\sqrt{\frac{10}{12}}|0\rangle + \sqrt{\frac{2}{12}}|1\rangle \big ) \otimes |0\rangle =: |\psi_1\rangle$$

Here $R_{y_1} := R_y(2\arccos \sqrt{\frac{10}{12}}) \otimes I$.

**Step 2:** Finish the preparation by putting the second qubit in the right state, applying controlled Ry gates with the first qubit as the control.

To get the first two terms right, you need to convert the terms

$$\big (\sqrt{\frac{10}{12}}|0\rangle \big) \otimes |0\rangle \overset{R_{y_2}}\rightarrow \frac{1}{\sqrt{12}} \big (3|00\rangle + |01\rangle \big) \text{  (2.a) }$$
and
$$\big (\sqrt{\frac{2}{12}}|1\rangle \big) \otimes |0\rangle \overset{R_{y_3}}\rightarrow \frac{1}{\sqrt{12}} \big (|10\rangle+|11\rangle \big) \text{  (2.b) }$$

**Step 2.a:** The transformation
$$\big (\sqrt{\frac{10}{12}}|0\rangle \big) \otimes |0\rangle \overset{R_{y_2}}\rightarrow \frac{1}{\sqrt{12}} \big (3|00\rangle + |01\rangle \big)$$

is equivalent to the following transformation (to see this, just multiply both sides by $\sqrt{\frac{12}{10}}$):

$$|0\rangle \otimes |0\rangle \overset{R_{y_2}}\rightarrow \frac{1}{\sqrt{10}} \big (3|00\rangle + |01\rangle \big) = |0\rangle \otimes \frac{1}{\sqrt{10}} \big (3|0\rangle + |1\rangle \big)$$

This rotation should only be applied if the first qubit is in state $|0\rangle$, i.e., we need a conditional-on-zero rotation. The rotation angle can be determined by $\cos\frac{\theta_2}{2} = \frac{3}{\sqrt{10}}$ and $\sin\frac{\theta_2}{2} = \frac{1}{\sqrt{10}}$.

**Step 2.b:** Similarly, the transformation

$$\big (\sqrt{\frac{2}{12}}|1\rangle \big) \otimes |0\rangle \overset{R_{y_3}}\rightarrow \frac{1}{\sqrt{12}} \big (|10\rangle+|11\rangle \big)$$

is equivalent to

$$|1\rangle \otimes |0\rangle \overset{R_{y_3}}\rightarrow \frac{1}{\sqrt{2}} \big (|10\rangle+|11\rangle \big)$$

and can be done using a controlled rotation, applied if first qubit is $|1\rangle$, i.e., a conditional-on-one rotation. The rotation angle can be determined by $\cos\frac{\theta_3}{2} = \frac{1}{\sqrt{2}}$ and $\sin\frac{\theta_3}{2} = \frac{1}{\sqrt{2}}$.

@[solution]({
    "id": "superposition__hardy_state_solution",
    "codePath": "./Solution.qs"
})
