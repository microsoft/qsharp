This task is quite different from the previous tasks in this section; 
at the first glance it might seem impossible to distinguish four different unitaries (i.e., get two bits of information) with just one unitary application! 

However, since the unitaries were chosen carefully (and you're not limited in the number of measurements you can do), it is possible. 
The solution uses the Bell states: the four orthogonal states which you can prepare by starting with the first of them $\frac{1}{\sqrt2}(\ket{00} + \ket{11})$ and applying the gates $I$, $X$, $Z$ and $Y$, respectively, to the first qubit. 
Thus the solution becomes: prepare the $\frac{1}{\sqrt2}(\ket{00} + \ket{11})$ state, apply the unitary and measure the resulting state in Bell basis to figure out which of the Bell states it is. See the Distinguish Quantum States kata, task 'Distinguish Four Bell states' for the details on how to do that.

@[solution]({
    "id": "distinguishing_unitaries__i_x_y_z_solution",
    "codePath": "Solution.qs"
})
