This task will also rely on counting $1$ bits in the bit string, and we'll use the same approach to it as we did in the previous task. However, this time the condition we need to check is different: instead of checking whether the number of $1$ bits is a fixed number, we want to check whether it's greater than a constant $\frac{N - 1} / 2$. Ideally, we want to do that without implementing the general logic of comparing an integer stored in a qubit array to a constant.

To find a shortcut for our case, let's see the bit strings for which we want to flip our target qubit for different values of $N$.

<table>
    <tr>
    <th>$N$</th>
    <th>Counts to mark</th>
    <th>Binary notations of counts <br/> in little endian</th>
    </tr>
    <tr>
    <td>$3$</td>
    <td>$2$ <br/> $3$</td>
    <td>$01$ <br/> $11$</td>
    </tr>
    <tr>
    <td>$5$</td>
    <td>$3$ <br/> $4$ <br/> $5$</td>
    <td>$110$ <br/> $001$ <br/> $101$</td>
    </tr>
    <tr>
    <td>$7$</td>
    <td>$4$ <br/> $5$ <br/> $6$ <br/> $7$</td>
    <td>$001$ <br/> $101$ <br/> $011$ <br/> $111$</td>
    </tr>
</table>

You can notice a pattern in the bit strings we need to mark for $N = 3$ and $N = 7$: they are all possibe bit strings with the most significant bit equal to $1$, so they can be marked really easily using a controlled $X$ gate with a single qubit as control - the qubit storing the most significant bit of the count number! 

Unfortunately, this pattern breaks for $N = 5$, since we need to mark the bit string $110$ with the most significant bit equal to $0$. But this is easy to fix with a bit of a hack: we just mark this bit string separately using a $CCNOT$ gate with two least significant bits as controls, taking advantage of the fact that this bit string is the only one that satisfies this condition.

> Keep in mind that this is a hack that relies on the small number of qubits in our input. In general case, we'd want to implement the proper logic for comparing the bit count with a constant (we'll learn to do it in one of the later katas).

@[solution]({
    "id": "marking_oracles__majority_solution",
    "codePath": "./Solution.qs"
})
