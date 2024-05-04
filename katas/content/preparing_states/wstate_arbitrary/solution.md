This problem allows a variety of solutions that rely on techniques from arbitrary rotations to recursion to postselection. 

### Iterative Solution

The first approach we will describe relies on performing a sequence of controlled rotations.

To prepare a weighted superposition $\cos \theta |0\rangle + \sin \theta |1\rangle$ on a single qubit, we need to start with the $|0\rangle$ state and apply the $R_y$ gate to it with the angle parameter equal to $2 \theta$. 
We'll apply the $R_y$ gate with angle $2 \theta_1 = 2\arcsin \frac{1}{\sqrt{N}}$ to the first qubit of the register to prepare the following state:

$$(\cos \theta_1 |0\rangle + \sin \theta_1 |1\rangle) \otimes |0 \dots 0\rangle = \frac{1}{\sqrt{N}}|10 \dots 0\rangle + \frac{\sqrt{N-1}}{\sqrt{N}}|00 \dots 0\rangle $$

The first term $\frac{1}{\sqrt{N}}|10 \dots 0\rangle$ already matches the first term of the $|W_N\rangle$ state; now we need to convert the second term $\frac{\sqrt{N-1}}{\sqrt{N}}|00 \dots 0\rangle$ into the rest of the $|W_N\rangle$ terms.

To prepare a term that matches the second term of the $|W_N\rangle$ state, we can apply another $R_y$ gate to the term $|00 \dots 0\rangle$, this time to the second qubit, with an angle $2 \theta_2 = 2\arcsin \frac{1}{\sqrt{N-1}}$.
To make sure it doesn't affect the term that we're already happy with, we will apply a controlled version of the $R_y$ gate, with the first qubit of the register in state $|0\rangle$ as control.
This will change our state to

$$\frac{1}{\sqrt{N}}|10 \dots 0\rangle + \frac{\sqrt{N-1}}{\sqrt{N}} |0\rangle \otimes (\cos \theta_2 |0\rangle + \sin \theta_2 |1\rangle) \otimes |0 \dots 0\rangle = $$
$$= \frac{1}{\sqrt{N}}|10 \dots 0\rangle + \frac{\sqrt{N-1}}{\sqrt{N}} \frac{1}{\sqrt{N-1}} |010 \dots 0\rangle + \frac{\sqrt{N-1}}{\sqrt{N}} \frac{\sqrt{N-2}}{\sqrt{N-1}} |000 \dots 0\rangle$$

Now we have two terms that match the terms of the $|W_N\rangle$ state, and need to convert the third term $\frac{\sqrt{N-2}}{\sqrt{N}}|00 \dots 0\rangle$ into the rest of terms.

We will keep going like this, preparing one term of the $|W_N\rangle$ state at a time, until the rotation on the last qubit will be an $X$ gate, controlled on all previous qubits being in the $|0 \dots 0\rangle$ state.

@[solution]({
    "id": "preparing_states__wstate_arbitrary_solution_a",
    "codePath": "./SolutionA.qs"
})

### Recursive Solution

We can express the same sequence of gates using recursion, if we notice that 

$$|W_N\rangle = \frac{1}{\sqrt{N}}|10 \dots 0\rangle + \frac{\sqrt{N-1}}{\sqrt{N}}|0\rangle \otimes |W_{N-1}\rangle$$

The first step of the solution would still be applying the $R_y$ gate with angle $2 \theta_1 = 2\arcsin \frac{1}{\sqrt{N}}$ to the first qubit of the register to prepare the following state:

$$\frac{1}{\sqrt{N}}|10 \dots 0\rangle + \frac{\sqrt{N-1}}{\sqrt{N}}|00 \dots 0\rangle $$

But we would express the rest of the controlled rotations as the operation that prepares the $|W_{N-1}\rangle$ state, controlled on the $|0\rangle$ state of the first qubit.

> Note that you don't have to implement the controlled version of this operation yourself; it is sufficient to add `is Adj + Ctl` to the signature of the operation `WState_Arbitrary` to specify that controlled variant has to be generated automatically.

@[solution]({
    "id": "preparing_states__wstate_arbitrary_solution_b",
    "codePath": "./SolutionB.qs"
})

### Post-selection Solution

Let's assume that we know how to prepare the $W$ state for $N = 2^k$ (we've discussed this in the previous task), and figure out how to use this knowledge as a building block for solving this task.

Let's look at the smallest possible case for which $N \neq 2^k$: $N = 3$ (we'll be able to generalize our solution for this case to an arbitrary number of qubits). The target $W$ state looks like this:  

$$|W_3\rangle = \frac{1}{3}\big(|100\rangle + |010\rangle + |001\rangle\big)$$

We will start by finding the smallest power of 2 $P$ which is greater than or equal to $N$; for our case $N = 3$ this power will be $P = 4$. We will allocate an extra $P - N$ qubits and use the solution of the previous task to prepare the $W_P$ state that looks as follows (with the state of the extra qubit highlighted in bold):  

$$|W_4\rangle = \frac{1}{2}\big |100\textbf{0}\rangle + |010\textbf{0}\rangle + |001\textbf{0}\rangle + |000\textbf{1}\rangle \big) = $$
$$= \frac{\sqrt3}{2} \cdot \frac{1}{\sqrt3}\big(|100\rangle + |010\rangle + |001\rangle \big) \otimes |\textbf{0}\rangle + \frac{1}{2}|000\rangle \otimes |\textbf{1}\rangle = $$
$$= \frac{\sqrt3}{2} |W_3\rangle \otimes |\textbf{0}\rangle + \frac{1}{2}|000\rangle \otimes |\textbf{1}\rangle$$

As we can see, if the extra qubit is in the $|0\rangle$ state, the main 3 qubits that we are concerned about are in the right $|W_3\rangle$ state. 

What happens if we measure just the extra qubit? This causes a partial collapse of the system to the state defined by the measurement result:
* If the result is $|0\rangle$, the system collapses to the $|W_3\rangle$ state - which is exactly what we wanted to achieve.
* If the result is $|1\rangle$, the system collapses to a state $|000\rangle$, so our goal is not achieved. The good thing is, this only happens in 25% of the cases, and we can just try again.

If we generalize this approach to an arbitrary $N$, we'll have 

$$|W_P\rangle = \frac{\sqrt{N}}{\sqrt{P}} |W_N\rangle \otimes |\textbf{0}\rangle^{\otimes P-N} + \frac{\sqrt{P-N}}{\sqrt{P}} |0\rangle^{\otimes N} \otimes |W_{P-N}\rangle$$

Measuring the extra $P-N$ qubits gives us two possibilities:
* All the extra qubits are in the $|0\rangle$ state; this means the main qubits collapse to the $|W_N\rangle$ state. 
* One of the extra qubits is in the $|1\rangle$ state; this means that the main qubits collapse to the $|0\rangle^{\otimes N}$ state, which is **not** the desired state. In this case we will reset and try again until all the extra qubits are in the $|0\rangle$ state.

@[solution]({
    "id": "preparing_states__wstate_arbitrary_solution_c",
    "codePath": "./SolutionC.qs"
})
