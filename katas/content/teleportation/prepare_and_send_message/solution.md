The exercise requires to prepare the `qMessage` is an eigenstate of the three Pauli basis.

- Initialize the `qMessage` qubit in $\ket{0}$ or $\ket{1}$ based on the value of state. These are the eigenstates of PauliZ basis.
- Using H and S gate, eigenstates of PauliX and PauliY can be created from eigenstates of PauliZ.
- Finally, use the SendMessage function prepared in Send Message exercise. 

@[solution]({
    "id": "teleportation__prepare_and_send_message_solution",
    "codePath": "./Solution.qs"
})