namespace Kata {
    operation PrepareAndSendMessage(qAlice : Qubit, basis : Pauli, state : Result) : (Bool, Bool) {
        use qMessage = Qubit();
        if state == One {
            X(qMessage);
        }
        if basis != PauliZ {
            H(qMessage);
        }
        if basis == PauliY {
            S(qMessage);
        }
        let classicalBits = SendMessage(qAlice, qMessage);
        Reset(qMessage);
        return classicalBits;
    }

    operation SendMessage(qAlice: Qubit, qMessage: Qubit) : (Bool, Bool) {
        CNOT(qMessage, qAlice);
        H(qMessage);
        return (M(qMessage) == One, M(qAlice) == One);
    }
}