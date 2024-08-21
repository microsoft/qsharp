There are multiple ways to prepare this state; the first solution described will focus on preparing this state without using arbitrary rotation gates. 

Initially, you'll prepare an equal superposition of all basis states on the two starting qubits by applying the $H$ gate to each of them: 
$$\frac{1}{2} \big(\ket{00} + \ket{01} + \ket{10} + \ket{11}\big)$$

This state is a superposition of the state you want to prepare, and the $\ket{11}$ state that you'd like to discard. 
You can do exactly that by performing the right measurement on the system. To do this, you allocate an extra qubit (sometimes referred to as an *ancilla* qubit). With this extra qubit, the new state becomes: 
$$\frac{1}{2} \big(\ket{00\textbf{0}} + \ket{01\textbf{0}} + \ket{10\textbf{0}} + \ket{11\textbf{0}}\big)$$

Now, you want to separate the first three basis states from the last one and to store this separation in the extra qubit. 
For example, you can keep the state of the extra qubit $\ket{0}$ for the basis states that you want to keep, and switch it to $\ket{1}$ for the basis states that you'd like to discard. 
A $CCNOT$ gate can be used to accomplish this, with the first two qubits used as control qubits and the extra qubit as target. 
When the gate is applied, the state of the extra qubit will only change to $\ket{1}$ if both control qubits are in the $\ket{11}$ state, which marks exactly the state that you want to discard:

$$CCNOT\frac{1}{2} \big(\ket{00\textbf{0}} + \ket{01\textbf{0}} + \ket{10\textbf{0}} + \ket{11\textbf{0}}\big) = 
\frac{1}{2}\big(\ket{00} + \ket{01} + \ket{10} \big) \otimes \ket{\textbf{0}} + \frac{1}{2}\ket{11} \otimes \ket{\textbf{1}} $$

Finally, you measure just the extra qubit; this causes a partial collapse of the system to the state defined by the measurement result:
* If the result is $\ket{0}$, the system collapses to a state that is a linear combination of basis states which had the extra qubit in state $\ket{0}$, that is, the two qubits end up in the target state $\frac{1}{\sqrt3}\big(\ket{00} + \ket{01} + \ket{10}\big)$. 
* If the result is $\ket{1}$, the system collapses to a state $\ket{11}$, so your goal isn't achieved. The good thing is, this only happens in $25\%$ of the cases, and you can just reset your qubits to the $\ket{00}$ state and try again.

> Q# has a built-in [repeat-until-success (RUS) loop](https://learn.microsoft.com/azure/quantum/user-guide/language/expressions/conditionalloops#repeat-expression), which comes in handy in this case. 
> * You'll describe the main operations (applying $H$ and $CCNOT$ gates and the measurement) in the `repeat` part of the loop, which specifies its body.  
> * `until` section specifies the condition which will break the loop. In this case, the result of the measurement needs to be `Zero` to indicate your success.  
> * Finally, the `fixup` section allows you to clean up the results of the loop body execution before trying again if the success criteria isn't met. In this case, you reset the first two qubits back to the $\ket{00}$ state.

This technique is sometimes called *post-selection*.

@[solution]({
    "id": "preparing_states__three_states_two_qubits_solution_a",
    "codePath": "./SolutionA.qs"
})

Alternatively, this state can be prepared using arbitrary rotations (the $R_y$ gate). 

To start, you'll try to find a decomposition of the target state that makes it easier to see how to prepare the state.  
Knowing that $\ket{+} = \frac{1}{\sqrt{2}}\big(\ket{0}+\ket{1}\big)$, you can represent the state as follows:

$$ \frac{1}{\sqrt{3}} \big(\ket{00} + \ket{01} + \ket{10}\big) = \frac{\sqrt{2}}{\sqrt{3}}\ket{0} \otimes \ket{+} + \frac{1}{\sqrt{3}}\ket{1} \otimes \ket{0} $$ 

To prepare this state, you first want to prepare the first qubit in the state $ \frac{\sqrt{2}}{\sqrt{3}}\ket{0} + \frac{1}{\sqrt{3}}\ket{1} $, while leaving the second qubit unchanged. 
To do this, you can use a rotation gate $R_y$ (see Single Qubit Gates kata) which will perform the following transformation:
$$ R_y\ket{0} = \cos\frac{\theta}{2}\ket{0} + \sin\frac{\theta}{2}\ket{1} $$
You need to find a value of $\theta$ which satisfies both: 
$$\cos\frac{\theta}{2} = \frac{\sqrt{2}}{\sqrt{3}} \text{ and } \sin\frac{\theta}{2} = \frac{1}{\sqrt{3}}$$

Solving the last equation for $\theta$ gives you $\frac{\theta}{2} = \arcsin\frac{1}{\sqrt{3}}$, or $\theta = 2 \arcsin\frac{1}{\sqrt{3}}$.

When you apply this to your first qubit, you'll get your desired intermediary state:
$$ R_y(2 \arcsin\frac{1}{\sqrt{3}})\ket{0} \otimes \ket{0} = 
\left(\frac{\sqrt{2}}{\sqrt{3}}\ket{0} + \frac{1}{\sqrt{3}}\ket{1} \right) \otimes \ket{0} = 
\frac{\sqrt{2}}{\sqrt{3}}\ket{0} \otimes \ket{0} + \frac{1}{\sqrt{3}}\ket{1} \otimes \ket{0}$$

Now, the second term of this state already matches your final goal, so you need to adjust the first term: 
prepare the $\ket{+}$ state on the second qubit only if the first qubit is in the $\ket{0}$ state. 
To do this, you apply a conditional $H$ gate to the second qubit, if the first qubit is in the $\ket{0}$ state, this will give your desired state:
$$ \frac{\sqrt{2}}{\sqrt{3}}\ket{0} \otimes \ket{+} + \frac{1}{\sqrt{3}}\ket{1} \otimes \ket{0}$$

> In Q#, you can apply a conditional gate with arbitrary controls using the [`ApplyControlledOnInt` operation](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonint). 
> In this case you want the $H$ gate to be applied if the control qubit is in the $\ket{0}$ state, so you'll use the `ApplyControlledOnInt(0, H)` gate.

@[solution]({
    "id": "preparing_states__three_states_two_qubits_solution_b",
    "codePath": "./SolutionB.qs"
})
