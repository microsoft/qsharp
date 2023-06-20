### Multi-qubit Pauli measurements
Joint measurement is a generalization of the measurement in the computational basis. 
Pauli measurements can also be generalized to a larger number of qubits. A multi-qubit Pauli measurement corresponds to an operator $M_1 \otimes \dotsc \otimes M_n$, with each $M_i$ being from the set of gates $\{X,Y,Z,I\}$. If at least one of the operators is not the identity matrix, then the measurement can result in two outcomes: a `Result` of `Zero` corresponding to eigenvalue $+1$ and a `Result` of `One` corresponding to the eigenvalue $-1$. The corresponding projection operators are the projections onto the corresponding eigenspaces.

For example, a Pauli/joint measurement corresponding to the $X\otimes Z$ operator can be characterized as follows:
<table style="border:1px solid">
    <col width=50>
    <col width=50>
    <col width=150>
    <col width=250>
    <tr>
        <th style="text-align:center; border:1px solid">Eigenvalue</th>
        <th style="text-align:center; border:1px solid">Measurement Result in Q#</th>
        <th style="text-align:center; border:1px solid">Eigenbasis</th>
        <th style="text-align:center; border:1px solid">Measurement Projector</th>
    </tr>
    <tr>
        <td style="text-align:center; border:1px solid">$+1$</td>
        <td style="text-align:center; border:1px solid">Zero</td>
        <td style="text-align:center; border:1px solid">$\{ \ket{0,+}, \ket{1,-} \}$</td>
        <td style="text-align:center; border:1px solid">$P_{+1} = \ket{0,+}\bra{0,+} +  \ket{1,-} \bra{1,-}$</td>
     </tr>
    <tr>
        <td style="text-align:center; border:1px solid">$-1$</td>
        <td style="text-align:center; border:1px solid">One</td>
        <td style="text-align:center; border:1px solid">$\{ \ket{0,-}, \ket{1,+} \}$</td>
        <td style="text-align:center; border:1px solid">$P_{-1} = \ket{0,-}\bra{0,-} +  \ket{1,+} \bra{1,+}$</td>
     </tr>
 </table>   
 
 The rules for measurements are then the same as those outlined in the [partial measurements section](#Partial-Measurements), with the projection operators in the table.

### <span style="color:blue">Exercise 10</span>: Parity measurement in different basis

Consider a system which is in a state $\alpha |00\rangle + \beta |01\rangle + \beta |10\rangle + \alpha |11\rangle$.

What are the possible outcomes and their associated probabilities, if a measurement in an $XX$ Pauli measurement is done?
