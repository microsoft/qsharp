### <span style="color:blue">Task 4.3</span>: Implement the meeting oracle

Suppose that you would like to schedule a meeting with your co-worker Jasmine. 
You both work five day workweeks, and $|x\rangle$ and $|jasmine\rangle$ are 5-bit states represent your and Jasmine's schedules. 
The schedules are indicators of a person being busy on that day: a $1$ bit means that person is busy on that day, and $0$ means they're free for a meeting that day. Implement a function that determines if you and Jasmine can schedule a meeting during the week, i.e., whether there is a day when both schedules have a $0$ simultaneously.

**Inputs:**

  1. 5 qubits in an arbitrary state $|x\rangle$ representing your schedule for the week (input/query register).
  2. 5 qubits in an arbitrary state $|jasmine\rangle$ representing Jasmine's schedule for the week (input/query register).
  3. A qubit in an arbitrary state $|y\rangle$ (target qubit).

**Goal:**

Flip the state of $|y\rangle$ if you and Jasmine are both free on the same day for at least one day during the week.  Recall that a $0$ means that a person is free on that day.

**Examples:**

* If $|x\rangle=|10101\rangle$ and $|jasmine\rangle=|01010\rangle$, do nothing (there is no day on which you both are free).
* If $|x\rangle=|10001\rangle$ and $|jasmine\rangle=|01010\rangle$, flip the state $|y\rangle$ (you are both free on Wednesday).
* If $|x\rangle=|00000\rangle$ and $|jasmine\rangle=|00000\rangle$, flip the state $|z\rangle$ (you are both free all week).
* If $|x\rangle=|11111\rangle$ and $|jasmine\rangle=|11111\rangle$, do nothing (you are both busy all week).
    
<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a marking oracle, because we are flipping the state of the target qubit $|y\rangle$ based on the state of the inputs $|x\rangle$ and $|jasmine\rangle$. Notice that even though we do not have the typical single-input-register situation that we saw earlier, this is still a marking oracle.
</details>
