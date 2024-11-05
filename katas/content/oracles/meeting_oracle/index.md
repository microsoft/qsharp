Suppose that you'd like to schedule a meeting with your co-worker Jasmine.
You both work five day workweeks, and $\ket{x}$ and $\ket{jasmine}$ are 5-bit states representing your and Jasmine's schedules.
The schedules are indicators of a person being busy on that day: a $1$ bit means that person is busy on that day, and $0$ means they're free for a meeting that day. Implement a function that determines if you and Jasmine can schedule a meeting during the week, that is, whether there is a day when both schedules have a $0$ simultaneously.

**Inputs:**

  1. 5 qubits in an arbitrary state $\ket{x}$ representing your schedule for the week (input/query register).
  2. 5 qubits in an arbitrary state $\ket{jasmine}$ representing Jasmine's schedule for the week (input/query register).
  3. A qubit in an arbitrary state $\ket{y}$ (target qubit).

**Goal:**

Flip the state of $\ket{y}$ if you and Jasmine are both free on the same day for at least one day during the week.  Recall that a $0$ means that a person is free on that day.

**Examples:**

* If $\ket{x}=\ket{10101}$ and $\ket{jasmine}=\ket{01010}$, do nothing (there is no day on which you both are free).
* If $\ket{x}=\ket{10001}$ and $\ket{jasmine}=\ket{01010}$, flip the state $\ket{y}$ (you are both free on Wednesday).
* If $\ket{x}=\ket{00000}$ and $\ket{jasmine}=\ket{00000}$, flip the state $\ket{y}$ (you are both free all week).
* If $\ket{x}=\ket{11111}$ and $\ket{jasmine}=\ket{11111}$, do nothing (you are both busy all week).
    
<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?</b></summary>
    This is a marking oracle, because you're flipping the state of the target qubit $\ket{y}$ based on the state of the inputs $\ket{x}$ and $\ket{jasmine}$. Notice that even though you don't have the typical single-input-register situation that we saw earlier, this is still a marking oracle.
</details>
