**Input:** A qubit which is guaranteed to be in one of the three states:
    
* $\ket{A} = \frac{1}{\sqrt{2}} \big( \ket{0} + \ket{1} \big)$,
* $\ket{B} = \frac{1}{\sqrt{2}} \big( \ket{0} + \omega \ket{1} \big)$,
* $\ket{C}= \frac{1}{\sqrt{2}} \big( \ket{0} + \omega^2 \ket{1} \big)$,
    
Here $\omega = e^{2i \pi/ 3}$.
    
**Output:** 
    
* 1 or 2 if the qubit was in the $\ket{A}$ state, 
* 0 or 2 if the qubit was in the $\ket{B}$ state, 
* 0 or 1 if the qubit was in the $\ket{C}$ state.
    
You are never allowed to give an incorrect answer. Your solution will be called multiple times, with one of the states picked with equal probability every time.
    
The state of the qubit at the end of the operation does not matter. 