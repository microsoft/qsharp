namespace Kata {
    operation SendMessage(qAlice : Qubit, qMessage : Qubit) : (Bool, Bool) {
        CNOT(qMessage, qAlice);
        H(qMessage);
        return (M(qMessage) == One, M(qAlice) == One);
    }
}