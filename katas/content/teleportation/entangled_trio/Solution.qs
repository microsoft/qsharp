namespace Kata {
    operation EntangleThreeQubits(qAlice : Qubit, qBob : Qubit, qCharlie : Qubit) : Unit is Adj {
        // now state is: 1/sqrt(2) (|000⟩ + |010⟩)
        H(qBob);
        // state: 1/sqrt(2) (|000⟩ + |011⟩)
        CNOT(qBob, qCharlie);
        // state: 1/2 (|000⟩ + |011⟩ + |100⟩ + |111⟩)
        H(qAlice);
        // final state:  1/2 (|000⟩ + |011⟩ + |101⟩ + |110⟩)
        CNOT(qAlice, qCharlie);
    }
}