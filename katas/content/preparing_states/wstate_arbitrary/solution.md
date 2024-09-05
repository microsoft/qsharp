This problem allows a variety of solutions that rely on techniques from arbitrary rotations to recursion to postselection. 

### Iterative Solution

The first approach relies on performing a sequence of controlled rotations.

To prepare a weighted superposition $\cos \theta \ket{0} + \sin \theta \ket{1}$ on a single qubit, you need to start with the $\ket{0}$ state and apply the $R_y$ gate to it with the angle parameter equal to $2 \theta$. 
You'll apply the $R_y$ gate with angle $2 \theta_1 = 2\arcsin \frac{1}{\sqrt{N}}$ to the first qubit of the register to prepare the following state:

$$(\cos \theta_1 \ket{0} + \sin \theta_1 \ket{1}) \otimes \ket{0 \dots 0} = \frac{1}{\sqrt{N}}\ket{10 \dots 0} + \frac{\sqrt{N-1}}{\sqrt{N}}\ket{00 \dots 0} $$

The first term $\frac{1}{\sqrt{N}}\ket{10 \dots 0}$ already matches the first term of the $\ket{W_N}$ state; now you need to convert the second term $\frac{\sqrt{N-1}}{\sqrt{N}}\ket{00 \dots 0}$ into the rest of the $\ket{W_N}$ terms.

To prepare a term that matches the second term of the $\ket{W_N}$ state, you can apply another $R_y$ gate to the term $\ket{00 \dots 0}$, this time to the second qubit, with an angle $2 \theta_2 = 2\arcsin \frac{1}{\sqrt{N-1}}$.
To make sure it doesn't affect the term that you're already happy with, you'll apply a controlled version of the $R_y$ gate, with the first qubit of the register in state $\ket{0}$ as control.
This will change the state to

$$\frac{1}{\sqrt{N}}\ket{10 \dots 0} + \frac{\sqrt{N-1}}{\sqrt{N}} \ket{0} \otimes (\cos \theta_2 \ket{0} + \sin \theta_2 \ket{1}) \otimes \ket{0 \dots 0} = $$
$$= \frac{1}{\sqrt{N}}\ket{10 \dots 0} + \frac{\sqrt{N-1}}{\sqrt{N}} \frac{1}{\sqrt{N-1}} \ket{010 \dots 0} + \frac{\sqrt{N-1}}{\sqrt{N}} \frac{\sqrt{N-2}}{\sqrt{N-1}} \ket{000 \dots 0}$$

Now you have two terms that match the terms of the $\ket{W_N}$ state, and need to convert the third term $\frac{\sqrt{N-2}}{\sqrt{N}}\ket{00 \dots 0}$ into the rest of terms.

You'll keep going like this, preparing one term of the $\ket{W_N}$ state at a time, until the rotation on the last qubit will be an $X$ gate, controlled on all previous qubits being in the $\ket{0 \dots 0}$ state.

@[solution]({
    "id": "preparing_states__wstate_arbitrary_solution_a",
    "codePath": "./SolutionA.qs"
})

### Recursive Solution

You can express the same sequence of gates using recursion, if you notice that 

$$\ket{W_N} = \frac{1}{\sqrt{N}}\ket{10 \dots 0} + \frac{\sqrt{N-1}}{\sqrt{N}}\ket{0} \otimes \ket{W_{N-1}}$$

The first step of the solution would still be applying the $R_y$ gate with angle $2 \theta_1 = 2\arcsin \frac{1}{\sqrt{N}}$ to the first qubit of the register to prepare the following state:

$$\frac{1}{\sqrt{N}}\ket{10 \dots 0} + \frac{\sqrt{N-1}}{\sqrt{N}}\ket{00 \dots 0} $$

But you'd express the rest of the controlled rotations as the operation that prepares the $\ket{W_{N-1}}$ state, controlled on the $\ket{0}$ state of the first qubit.

> Note that you don't have to implement the controlled version of this operation yourself; it's sufficient to add `is Adj + Ctl` to the signature of the operation `WState_Arbitrary` to specify that controlled variant has to be generated automatically.

@[solution]({
    "id": "preparing_states__wstate_arbitrary_solution_b",
    "codePath": "./SolutionB.qs"
})

### Post-selection Solution

Let's assume that you know how to prepare the $W$ state for $N = 2^k$ (see the previous task), and figure out how to use this knowledge as a building block for solving this task.

Let's look at the smallest possible case for which $N \neq 2^k$: $N = 3$ (you'll be able to generalize the solution for this case to an arbitrary number of qubits). The target $W$ state looks like this:  

$$\ket{W_3} = \frac{1}{3}\big(\ket{100} + \ket{010} + \ket{001}\big)$$

You'll start by finding the smallest power of 2 $P$ which is greater than or equal to $N$; for this case $N = 3$, so this power will be $P = 4$. You'll allocate an extra $P - N$ qubits and use the solution of the previous task to prepare the $W_P$ state that looks as follows (with the state of the extra qubit highlighted in bold):  

$$\ket{W_4} = \frac{1}{2}\big( \ket{100\textbf{0}} + \ket{010\textbf{0}} + \ket{001\textbf{0}} + \ket{000\textbf{1}} \big) = $$
$$= \frac{\sqrt3}{2} \cdot \frac{1}{\sqrt3}\big(\ket{100} + \ket{010} + \ket{001} \big) \otimes \ket{\textbf{0}} + \frac{1}{2}\ket{000} \otimes \ket{\textbf{1}} = $$
$$= \frac{\sqrt3}{2} \ket{W_3} \otimes \ket{\textbf{0}} + \frac{1}{2}\ket{000} \otimes \ket{\textbf{1}}$$

As you can see, if the extra qubit is in the $\ket{0}$ state, the main 3 qubits that you're concerned about are in the right $\ket{W_3}$ state. 

What happens if you measure just the extra qubit? This causes a partial collapse of the system to the state defined by the measurement result:
* If the result is $\ket{0}$, the system collapses to the $\ket{W_3}$ state - which is exactly what you wanted to achieve.
* If the result is $\ket{1}$, the system collapses to a state $\ket{000}$, so your goal is not achieved. The good thing is, this only happens in $25\%$ of the cases, and you can just try again.

If you generalize this approach to an arbitrary $N$, you'll have 

$$\ket{W_P} = \frac{\sqrt{N}}{\sqrt{P}} \ket{W_N} \otimes \ket{\textbf{0}}^{\otimes P-N} + \frac{\sqrt{P-N}}{\sqrt{P}} \ket{0}^{\otimes N} \otimes \ket{W_{P-N}}$$

Measuring the extra $P-N$ qubits gives you two possibilities:
* All the extra qubits are in the $\ket{0}$ state; this means the main qubits collapse to the $\ket{W_N}$ state. 
* One of the extra qubits is in the $\ket{1}$ state; this means that the main qubits collapse to the $\ket{0}^{\otimes N}$ state, which is **not** the desired state. In this case, you'll reset and try again until all the extra qubits are in the $\ket{0}$ state.

@[solution]({
    "id": "preparing_states__wstate_arbitrary_solution_c",
    "codePath": "./SolutionC.qs"
})
