The exercise aims to prepare the `qMessage` as one of the basis states of the required Pauli bases.

- Initialize the `qMessage` qubit in $\ket{0}$ or $\ket{1}$ based on the value of `state`. These are the eigenstates of Pauli $Z$ gate.
- Using the $H$ gate, eigenstates of the Pauli $X$ gate can be prepared from the eigenstates of Pauli $Z$:
  $$\ket{+} = H\ket{0}, \ket{-} = H\ket{1}$$
- Using the $S$ gate, eigenstates of the Pauli $Y$ gate can be prepared from the eigenstates of Pauli $X$:
  $$\ket{i} = S\ket{+}, \ket{-i} = S\ket{-}$$

Finally, use the `SendMessage` operation implemented in the earlier exercise to send the message. 

@[solution]({
    "id": "teleportation__prepare_and_send_message_solution",
    "codePath": "./Solution.qs"
})