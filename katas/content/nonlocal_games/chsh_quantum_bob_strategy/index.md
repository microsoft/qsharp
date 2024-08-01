**Inputs:**

- Bob's starting bit (Y).
- Bob's half of Bell pair he shares with Alice.

**Goal:**
Measure Bob's qubit in the $\frac{\pi}{8}$ basis if his bit is 0 (false), or the $-\frac{\pi}{8}$ basis
if his bit is 1 (true) and return the measurement result as a Boolean value: map `Zero` to false and `One` to true.
The state of the qubit after the operation does not matter.

Measuring a qubit in the $\theta$ basis is the same as rotating the qubit by $\theta$, clockwise, and then making a standard measurement in the Z basis.
