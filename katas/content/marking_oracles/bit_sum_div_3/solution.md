Let's start by considering a simpler version of the problem. How would you check whether the number of bits equal
to $1$ is divisible by $2$? 

You can just iterate over the input qubits and do a sequence of $CNOT$ gates with each of them as
the control and the output qubit as the target. The fact that $CNOT$ does a controlled flip of the state makes sure that
this is the same as taking sum of the bits modulo $2$, so in the end you just flip it again to become $1$ if the sum modulo
$2$ was $0$.

You can use the same approach here, but you need to implement addition modulo $3$ as an elementary operation.
To do this, you use a counter with two qubits in it (storing `sum` qubit first and `carry` second) and figure out the rules of updating them in a way
which allows to go from $\ket{0_{sum}0_{carry}}$ to $\ket{1_{sum}0_{carry}}$ to $\ket{0_{sum}1_{carry}}$ to $\ket{0_{sum}0_{carry}}$ with each increment. (The basis state $\ket{1_{sum}1_{carry}}$ remains unchanged to keep the transformation unitary, though we'll never use it in our solution.) One way to do this is as follows:

1. Start by updating the `sum` bit.  
   The `sum` changes if `carry` is $0$, and remains unchanged otherwise. We can implement this using controlled-on-zero $X$ gate.
2. Then, update the `carry` bit.
   It changes if the updated `sum` bit is $0$, which can be done using another controlled-on-zero $X$ gate.

With `IncrementMod3` operation implemented, you iterate over the input qubits and count the number of $1$ bits among them modulo $3$. If both counter qubits end up in $\ket{0}$ state, the number of $1$ bits is divisible by $3$, and you need to flip the target qubit.

@[solution]({
    "id": "marking_oracles__bit_sum_div_3_solution",
    "codePath": "./Solution.qs"
})
