The target state written in binary notation instead of integers can have its terms grouped in fours that look as follows:

$$ \ket{...00} + \ket{...01} - \ket{...10} - \ket{...11} $$

The states which have $1$ as their second-least significant bit have a $-1$ relative phase, and the states with $0$ in that bit have a $1$ relative phase. Written as a tensor product of single-qubit states, the target state looks as follows:

$$\frac1{\sqrt{2^n}} (\ket{0} + \ket{1}) \otimes (\ket{0} + \ket{1}) \otimes ... \otimes (\ket{0} - \ket{1}) \otimes (\ket{0} + \ket{1})$$

You could try to prepare this state by setting only $j_2$ to $1$ and the rest of bits of $j$ to $0$ and then applying the QFT. This will add the required relative phase of $-1$ on the second-to-last qubit bit, but will also add a relative phase of $i$ on the last qubit bit:

$$QFT(\ket{010...0}) = \frac1{\sqrt{2^n}} (\ket{0} + \ket{1}) \otimes (\ket{0} + \ket{1}) \otimes ... \otimes (\ket{0} - \ket{1}) \otimes (\ket{0} + i\ket{1})$$

To fix this last qubit state, you need to cancel out that last $i$ relative phase in some form. This can be achieved by using the following state as the input to QFT:

$$\frac1{\sqrt2} \big(e^{-i\pi/4} \ket{010...0} + e^{i\pi/4} \ket{110...0}\big)$$

You can write out the result of applying QFT to this state as follows, using the fact that $e^{\pm i\pi/4} = \frac{1 \pm i}{\sqrt2}$:

$$ QFT\bigg(\frac1{\sqrt2} \big(e^{-i\pi/4} \ket{010...0} + e^{i\pi/4} \ket{110...0}\big)\bigg) = \\

= \frac1{\sqrt2}\bigg( \frac{1-i}{\sqrt2} QFT\ket{010...0} + \frac{1+i}{\sqrt2} QFT\ket{110...0} \bigg) = \\

= \frac{(1-i)}{2\sqrt{2^n}} (\ket{0} + \ket{1}) \otimes (\ket{0} + \ket{1}) \otimes ... \otimes (\ket{0} - \ket{1}) \otimes (\ket{0} + i\ket{1}) + \\

+ \frac{(1+i)}{2\sqrt{2^n}} (\ket{0} + \ket{1}) \otimes (\ket{0} + \ket{1}) \otimes ... \otimes (\ket{0} - \ket{1}) \otimes (\ket{0} - i\ket{1}) = \\

= \frac{1}{\sqrt{2^{n}}} (\ket{0} + \ket{1}) \otimes (\ket{0} + \ket{1}) \otimes ... \otimes (\ket{0} - \ket{1}) \otimes (\ket{0} + \ket{1})$$
 
Creating the required initial superposition state can be done using the $T$ gate and its adjoint in addition to the more standard $X$ and $H$ gates:

1. Apply an $H$ gate to the first qubit and an $X$ gate to the second qubit:
   $$\frac1{\sqrt2} \big(\ket{010...0} + \ket{110...0}\big)$$
2. Add the relative phase $e^{i\pi/4}$ to the second term by applying a $T$ gate to the first qubit:
   $$\frac1{\sqrt2} \big(\ket{010...0} + e^{i\pi/4} \ket{110...0}\big)$$
3. Add the relative phase $e^{-i\pi/4}$ to the first term by applying an adjoint $T$ gate wrapped in $X$ gates to the first qubit (this way, the relative phase will be applied to $\ket{0}$ basis state of that qubit):
   $$\frac1{\sqrt2} \big(e^{-i\pi/4} \ket{010...0} + e^{i\pi/4} \ket{110...0}\big)$$


@[solution]({
    "id": "qft__square_wave_solution",
    "codePath": "./Solution.qs"
})
