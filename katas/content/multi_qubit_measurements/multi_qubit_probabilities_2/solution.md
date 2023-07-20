### Analytical solution

Using the expressions $|0\rangle = \frac{1}{\sqrt{2}} \big( |+\rangle + |-\rangle \big)$ and $|1\rangle = \frac{1}{\sqrt{2}} \big( |+\rangle - |-\rangle \big)$, we first express $|\psi\rangle$ in the Pauli X basis. This gives us
$$\ket \psi =  \frac{2}{3}\ket {00} + \frac{1}{3} \ket {01} + \frac{2}{3}\ket {11} = $$ 

$$= \frac{2}{3} \big[ \frac{1}{\sqrt{2}}\big(\ket{+} + \ket{-}\big) \otimes \frac{1}{\sqrt{2}} \big(\ket{+} + \ket{-}\big) \big] + $$ 
$$+ \frac{1}{3} \big[ \frac{1}{\sqrt{2}}\big(\ket{+} + \ket{-}\big) \otimes \frac{1}{\sqrt{2}} \big(\ket{+} - \ket{-}\big) \big] + $$ 
$$+ \frac{2}{3} \big[ \frac{1}{\sqrt{2}}\big(\ket{+} - \ket{-}\big) \otimes \frac{1}{\sqrt{2}} \big(\ket{+} - \ket{-}\big) \big] = $$ 

$$= \frac{1}{3} \big[ \big(\ket{+} + \ket{-}\big) \otimes \big(\ket{+} + \ket{-}\big) \big] + $$ 
$$+ \frac{1}{6} \big[ \big(\ket{+} + \ket{-}\big) \otimes \big(\ket{+} - \ket{-}\big) \big] + $$ 
$$+ \frac{1}{3} \big[ \big(\ket{+} - \ket{-}\big) \otimes \big(\ket{+} - \ket{-}\big) \big] = $$ 

$$= \frac{1}{3} \big[ \ket{++} + \ket{+-} + \ket{-+} + \ket{--} \big] + $$ 
$$+ \frac{1}{6} \big[ \ket{++} - \ket{+-} + \ket{-+} - \ket{--} \big] + $$ 
$$+ \frac{1}{3} \big[ \ket{++} - \ket{+-} - \ket{-+} + \ket{--} \big] = $$ 

$$= (\frac{1}{3} + \frac{1}{6} + \frac{1}{3})\ket{++} + $$
$$+ (\frac{1}{3} - \frac{1}{6} - \frac{1}{3})\ket{+-} + $$
$$+ (\frac{1}{3} + \frac{1}{6} - \frac{1}{3})\ket{-+} + $$
$$+ (\frac{1}{3} - \frac{1}{6} + \frac{1}{3})\ket{--} = $$

$$= \frac{5}{6}\ket{++} - \frac{1}{6}\ket{+-} + \frac{1}{6}\ket{-+} + \frac{1}{2}\ket{--} ;$$
After this, the probabilities of measuring each of the four basis vectors is given by the square of the absolute value of its amplitude in the superposition:
<table style="border:1px solid">
    <col width=150>
    <col width=150>
    <tr>
        <th style="text-align:center; border:1px solid">Measurement outcome</th>
        <th style="text-align:center; border:1px solid">Probability of outcome</th>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid">$++$</td>
        <td style="text-align:center; border:1px solid">$\left( \frac{5}{6}\right)^2 = \frac{25}{36}$</td>
    </tr> 
    <tr>
        <td style="text-align:center; border:1px solid">$+-$</td>
        <td style="text-align:center; border:1px solid">$\left( -\frac{1}{6}\right)^2 = \frac{1}{36}$</td>
    </tr> 
    <tr>
        <td style="text-align:center; border:1px solid">$-+$</td>
        <td style="text-align:center; border:1px solid">$\left( \frac{1}{6}\right)^2 = \frac{1}{36}$</td>
    </tr>     
    <tr>
        <td style="text-align:center; border:1px solid">$--$</td>
        <td style="text-align:center; border:1px solid">$\left( \frac{1}{2}\right)^2 = \frac{1}{4}$</td>
    </tr> 
</table>

### Code-based solution

We can also use Q# to solve this problem. It can be achieved in three steps:
1. Prepare the state $\ket \psi$.
2. Apply a transformation that maps the 2-qubit Pauli X basis into the 2-qubit computational basis. This transformation just applies a Hadamard gate to each of the qubits.
3. View probabilities of each basis state with `DumpMachine` function. Thanks to the previous step the following state equivalence holds:

<table style="border:1px solid">
    <col width=150>
    <col width=150>
    <tr>
        <th style="text-align:center; border:1px solid">Before basis transformation</th>
        <th style="text-align:center; border:1px solid">After basis transformation</th>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid">$\ket {++}$</td>
        <td style="text-align:center; border:1px solid">$\ket {00}$</td>
    </tr> 
    <tr>
        <td style="text-align:center; border:1px solid">$\ket {+-}$</td>
        <td style="text-align:center; border:1px solid">$\ket {01}$</td>
    </tr> 
    <tr>
        <td style="text-align:center; border:1px solid">$\ket {-+}$</td>
        <td style="text-align:center; border:1px solid">$\ket {10}$</td>
    </tr>     
    <tr>
        <td style="text-align:center; border:1px solid">$\ket {--}$</td>
        <td style="text-align:center; border:1px solid">$\ket {11}$</td>
    </tr> 
</table>

So the amplitudes of the computational basis states after the transformation are the same as the amplitudes of the basis states of the Pauli X basis before the transformation!

>To implement the first step, we can represent $\ket \psi$ as  
>$$\frac 2 3 \ket {00} + {\big (} \frac 1 {\sqrt 5} \ket {0} + \frac 2 {\sqrt 5} \ket {1} {\big )} \frac {\sqrt 5} 3 \ket {1}$$
>This representation tells us how we should rotate individual qubits. You can read more about preparing superposition states in the [Superposition kata](../../Superposition/Superposition.ipynb#Part--II.-Arbitrary-rotations.).
>
> Notice that we start by rotating the second qubit, as this gives a simpler implementation. If we started by rotating the first qubit, we would need to use a CNOT gate and a controlled $R_y$ gate to achieve the same result.

@[example]({
"id": "multi_qubit_probabilities_2_example",
"codePath": "solution.qs"
})
