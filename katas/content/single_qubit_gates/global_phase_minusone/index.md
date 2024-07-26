**Input:** A qubit in state $\ket{\psi} = \beta \ket{0} + \gamma \ket{1}$.

**Goal**: Change the state of the qubit to $- \beta \ket{0} - \gamma \ket{1}$.

> This change on its own is not observable - there's no experiment you can do on a standalone qubit to figure out whether it acquired the global phase or not.
> However, you can use a controlled version of this operation to observe the global phase it introduces.
> This is used in later katas as part of more complicated tasks.
