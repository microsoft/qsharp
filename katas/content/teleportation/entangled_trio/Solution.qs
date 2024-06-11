namespace Kata {
    operation EntangleThreeQubits(qAlice : Qubit, qBob : Qubit, qCharlie : Qubit) : Unit is Adj {
        H(qBob);
        CNOT(qBob, qCharlie);
        H(qAlice);
        CNOT(qAlice, qCharlie);
    }
}