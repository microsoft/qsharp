You start with the state $\ket{\psi_0}=\ket{00}$.

**Step 1:** Start by putting the first qubit in the state $\alpha\ket{0} + \beta\ket{1}$, where $\alpha$ and $\beta$ are the square roots of relative weights of all basis states which start with 0 and with 1, respectively.

In case of Hardy state, two basis states start with 0: $\frac{1}{\sqrt{12}} \big (3\ket{00} + \ket{01}\big )$ and two basis states start with 1: $\frac{1}{\sqrt{12}} \big (\ket{10} + \ket{11}\big )$.
The relative weights of 0 and 1 are just the sums of squares of their amplitudes:
$\alpha^2 = \frac{9}{12} + \frac{1}{12} = \frac{10}{12}$ and $\beta^2 = \frac{1}{12} + \frac{1}{12} = \frac{2}{12}$, respectively.
So you'll need to put the first qubit in the state $\sqrt{\frac{10}{12}}\ket{0} + \sqrt{\frac{2}{12}}\ket{1}$ using the $R_y$ gate:

$$\ket{00} \overset{R_{y_1}}\rightarrow \big (\sqrt{\frac{10}{12}}\ket{0} + \sqrt{\frac{2}{12}}\ket{1} \big ) \otimes \ket{0} =: \ket{\psi_1}$$

Here $R_{y_1} := R_y(2\arccos \sqrt{\frac{10}{12}}) \otimes I$.

**Step 2:** Finish the preparation by putting the second qubit in the right state, applying controlled $R_y$ gates with the first qubit as the control.

To get the first two terms right, you need to convert the terms

$$\big (\sqrt{\frac{10}{12}}\ket{0} \big) \otimes \ket{0} \overset{R_{y_2}}\rightarrow \frac{1}{\sqrt{12}} \big (3\ket{00} + \ket{01} \big) \text{  (2.a) }$$
and
$$\big (\sqrt{\frac{2}{12}}\ket{1} \big) \otimes \ket{0} \overset{R_{y_3}}\rightarrow \frac{1}{\sqrt{12}} \big (\ket{10}+\ket{11} \big) \text{  (2.b) }$$

**Step 2.a:** The transformation
$$\big (\sqrt{\frac{10}{12}}\ket{0} \big) \otimes \ket{0} \overset{R_{y_2}}\rightarrow \frac{1}{\sqrt{12}} \big (3\ket{00} + \ket{01} \big)$$

is equivalent to the following transformation (to see this, just multiply both sides by $\sqrt{\frac{12}{10}}$):

$$\ket{0} \otimes \ket{0} \overset{R_{y_2}}\rightarrow \frac{1}{\sqrt{10}} \big (3\ket{00} + \ket{01} \big) = \ket{0} \otimes \frac{1}{\sqrt{10}} \big (3\ket{0} + \ket{1} \big)$$

This rotation should only be applied if the first qubit is in state $\ket{0}$, that is, you need a conditional-on-zero rotation. The rotation angle can be determined by $\cos\frac{\theta_2}{2} = \frac{3}{\sqrt{10}}$ and $\sin\frac{\theta_2}{2} = \frac{1}{\sqrt{10}}$.

**Step 2.b:** Similarly, the transformation

$$\big (\sqrt{\frac{2}{12}}\ket{1} \big) \otimes \ket{0} \overset{R_{y_3}}\rightarrow \frac{1}{\sqrt{12}} \big (\ket{10}+\ket{11} \big)$$

is equivalent to

$$\ket{1} \otimes \ket{0} \overset{R_{y_3}}\rightarrow \frac{1}{\sqrt{2}} \big (\ket{10}+\ket{11} \big)$$

and can be done using a controlled rotation, applied if first qubit is $\ket{1}$, that is, a conditional-on-one rotation. The rotation angle can be determined by $\cos\frac{\theta_3}{2} = \frac{1}{\sqrt{2}}$ and $\sin\frac{\theta_3}{2} = \frac{1}{\sqrt{2}}$.

@[solution]({
    "id": "preparing_states__hardy_state_solution",
    "codePath": "./Solution.qs"
})
