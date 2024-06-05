namespace Kata {
    operation PrepareAndSendMessage(qAlice : Qubit, basis : Pauli, state : Result) : (Bool, Bool) {
        // Implement your solution here...
        return (false, false);
    }
    
    // You might find this helper operation from earlier tasks useful.
    operation SendMessage(qAlice: Qubit, qMessage: Qubit) : (Bool, Bool) {
        CNOT(qMessage, qAlice);
        H(qMessage);
        return (M(qMessage) == One, M(qAlice) == One);
    }
}