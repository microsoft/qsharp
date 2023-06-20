### Solution

A measurement outcome of $0$ on the first qubit corresponds to the projection operator $P_0 = |0\rangle\langle 0| \otimes \mathbb{1}$. Applying it to the state $\ket \psi$ gives us 
$$\big|P_0 \ket{\psi}\big|^2 = \big|\frac{1}{\sqrt{12}} \left(3\ket {00} + \ket{01}\right) \big|^2 = \frac{5}{6}$$
and 
$$\frac{P_0 \ket{\psi}}{\big|P_0 \ket{\psi}\big|} = \frac{1}{\sqrt{10}} \left( 3\ket{00} + \ket{01}\right)$$

Similarly, $P_1 = |1\rangle \langle 1 | \otimes \mathbb{1}$ is the projector corresponding to a measurement outcome of $1$ on the first qubit. Applying $P_1$ on $\ket{\psi}$ gives us $\big|P_1 \ket{\psi}\big|^2 = \frac{1}{6}$ and 

$$\frac{P_1 \ket{\psi}}{\big|P_1 \ket{\psi}\big|} = \frac{1}{\sqrt{2}} \left(\ket{10} + \ket{11}\right)$$

<table style="border:1px solid">
    <col width=150>
    <col width=150>
    <col width=150>
    <tr>
        <th style="text-align:center; border:1px solid">Measurement outcome</th>
        <th style="text-align:center; border:1px solid">Probability of outcome</th>
        <th style="text-align:center; border:1px solid">Post-measurement state</th>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid">$0$</td>
        <td style="text-align:center; border:1px solid">$\frac{5}{6}$</td>
        <td style="text-align:center; border:1px solid">$\frac{1}{\sqrt{10}} \left( 3\ket{00} + \ket{01}\right)$</td>
    </tr> 
    <tr>
        <td style="text-align:center; border:1px solid">$1$</td>
        <td style="text-align:center; border:1px solid">$\frac{1}{6}$</td>
        <td style="text-align:center; border:1px solid">$\frac{1}{\sqrt{2}} \left(\ket{10} + \ket{11}\right)$</td>
    </tr> 
</table>
