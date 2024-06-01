The exercise requires to prepare the `qMessage` is an eigenstate of the three Pauli basis.

- Initialize the `qMessage` qubit in $\ket{0}$ or $\ket{1}$ based on the value of state. These are the eigenstates of PauliZ basis.
- Using H, eigenstates of PauliX can be created from eigenstates of PauliZ. $\ket{+} = H\ket{0}$ and $\ket{-} = H\ket{1}$.
- Using S, eigenstates of PauliY can be created from eigenstates of PauliX. $\ket{i} = S\ket{+}$ and $\ket{-i} = S\ket{-}$.
- Finally, use the SendMessage function prepared in previous exercise. 

@[solution]({
    "id": "teleportation__prepare_and_send_message_solution",
    "codePath": "./Solution.qs"
})