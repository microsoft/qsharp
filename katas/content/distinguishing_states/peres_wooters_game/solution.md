> The task is a game inspired by a quantum detection problem due to Holevo ("Information-theoretical aspects of quantum measurement", A. Holevo) and Peres/Wootters ("Optimal detection of quantum information", A. Peres and W. K. Wootters). In the game, player A thinks of a number (0, 1 or 2) and the opponent, player B, tries to guess any number but the one chosen by player A. 
>
> Classically, if you just made a guess, you'd have to ask two questions to be right $100\%$ of the time. If instead, player A prepares a qubit with 0, 1, or 2 encoded into three single qubit states that are at an angle of 120 degrees with respect to each other and then hands the state to the opponent, then player B can apply a Positive Operator Valued Measure (POVM) consisting of 3 states that are perpendicular to the states chosen by player A. 
> It can be shown that this allows B to be right $100\%$ of the time with only 1 measurement, which is something that is not achievable with a von Neumann measurement on 1 qubit.
See also ("Quantum Theory:  Concepts and Methods", A. Peres) for a nice description of the optimal POVM.
    
Next, we address how we can implement the mentioned POVM by way of a von Neumann measurement, and then how to implement said von Neumann measurement in Q\#. First, we note that the POVM elements are given by the columns of the following matrix: 
     
$$M = \frac{1}{\sqrt{2}}\left(\begin{array}{rrr}
1 & 1 & 1 \\ 
1 & \omega & \omega^2 
\end{array}
\right)$$
    
where $\omega = e^{2 \pi i/3}$ denotes a primitive third root of unity. Our task will be to implement the rank 1 POVM given by the columns of $M$ via a von Neumann measurement. This can be done by \"embedding\" $M$ into a larger unitary matrix (taking complex conjugate and transpose):
    
$$M' = \frac{1}{\sqrt{3}}\left(\begin{array}{cccc}
1 & -1 & 1 & 0 \\ 
1 & -\omega^2 & \omega & 0 \\
1 & -\omega & \omega^2 & 0 \\
0 & 0 & 0 & -i\sqrt{3} 
\end{array}
\right)$$
    
Notice that applying $M'$ to input states given by column $i$ of $M$ (padded with two zeros to make it a vector of length $4$), where $i=0, 1, 2$ will never return the label $i$ as the corresponding vectors are perpendicular. 
    
We are therefore left with the problem of implementing $M'$ as a sequence of elementary quantum gates. Notice that 
    
$$M' \cdot {\rm diag}(1,-1,1,-1) = M' \cdot (\mathbf{1}_2 \otimes Z) = 
\frac{1}{\sqrt{3}}\left(\begin{array}{cccc}
1 & 1 & 1 & 0 \\ 
1 & \omega^2 & \omega & 0 \\
1 & \omega & \omega^2 & 0 \\
0 & 0 & 0 & i\sqrt{3} 
\end{array}
\right)$$
    
Using a technique used in the Rader (also sometimes called Rader-Winograd) decomposition of the discrete Fourier transform ("Discrete Fourier transforms when the number of data samples is prime", C. M. Rader), which reduces it to a cyclic convolution, we apply a $2\times 2$ Fourier transform on the indices $i,j=1,2$ of this matrix (i.e. a block matrix which consists of a direct sum of blocks $\mathbf{1}_1$, $H$, and $\mathbf{1}_1$ which we abbreviate in short as ${\rm diag}(1,H,1)$). 
    
> To implement this in Q#, we can use the following sequence of gates, applied to a 2-qubit array:
>
> ```
> CNOT(qs[1], qs[0]);
> Controlled H([qs[0]], qs[1]);
> CNOT(qs[1], qs[0]);
> ```
    
This yields
    
$${\rm diag}(1, H, 1) \cdot M' \cdot (\mathbf{1}_2 \otimes Z) \cdot {\rm diag}(1, H, 1) = 
\left(\begin{array}{rrrr}
\frac{1}{\sqrt3} & \sqrt{\frac23} & 0 & 0 \\ 
\sqrt{\frac23} & -\frac{1}{\sqrt3} & 0 & 0 \\
0 & 0 & i & 0 \\
0 & 0 & 0 & i 
\end{array}
\right)$$
    
This implies that after multiplication with the diagonal operator $(S^\dagger \otimes \mathbf{1}_2)$, we are left with 
    
$${\rm diag}(1, H, 1) \cdot M' \cdot (\mathbf{1}_2 \otimes Z) \cdot {\rm diag}(1, H, 1)\cdot (S^\dagger \otimes \mathbf{1}_2) = 
\left(\begin{array}{rrrr}
\frac{1}{\sqrt3} & \sqrt{\frac23} & 0 & 0 \\ 
\sqrt{\frac23} & -\frac{1}{\sqrt3} & 0 & 0 \\
0 & 0 & 1 & 0 \\
0 & 0 & 0 & 1 
\end{array}
\right)$$
    
which is a zero-controlled rotation $R$ around the $Y$-axis by an angle given by $\arccos \sqrt{\frac23}$ (plus reordering of rows and columns). 
    
> In Q#, we can implement this matrix as the following sequence of gates, applied to a 2-qubit array:
> ```
> CNOT(qs[1], qs[0]);
> X(qs[0]);
> let alpha = ArcCos(Sqrt(2.0 / 3.0));
> (ControlledOnInt(0, Ry))([qs[1]], (-2.0 * alpha, qs[0]));
> ```
    
Putting everything together, we can implement the matrix $M'$ by applying the inverses of gates:
    
$$M' = {\rm diag}(1,H,1) \cdot R \cdot (S \otimes \mathbf{1}_2) \cdot {\rm diag}(1,H,1) \cdot (\mathbf{1}_2 \otimes Z)$$
    
Noting finally, that to apply this sequence of unitaries to a column vector, we have to apply it in reverse when writing it as a program (as actions on vectors are left-associative).
    
@[solution]({
    "id": "distinguishing_states__peres_wooters_game_solution",
    "codePath": "Solution.qs"
})