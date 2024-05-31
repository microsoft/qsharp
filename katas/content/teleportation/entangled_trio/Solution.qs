namespace Kata {
    operation EntangleThreeQubits(qAlice : Qubit, qBob : Qubit, qCharlie : Qubit) : Unit is Adj {
        // Starting with |000⟩
        H(qBob); // now state is: 1/sqrt(2) (|000⟩ + |010⟩)
        CNOT(qBob, qCharlie); // state: 1/sqrt(2) (|000⟩ + |011⟩)
        H(qAlice); // state: 1/2 (|000⟩ + |011⟩ + |100⟩ + |111⟩)
        CNOT(qAlice, qCharlie); // final state:  1/2 (|000⟩ + |011⟩ + |101⟩ + |110⟩)
    }
}