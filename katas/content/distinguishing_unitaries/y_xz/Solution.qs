namespace Kata {
    operation DistinguishYfromXZ (unitary : (Qubit => Unit is Adj+Ctl)) : Int {
        use qs = Qubit[2];
        // prep (|0⟩ + |1⟩) ⊗ |0⟩
        within { H(qs[0]); }
        apply {  
            Controlled unitary(qs[0..0], qs[1]);
            Controlled unitary(qs[0..0], qs[1]);
        }

        // 0 means it was Y
        return M(qs[0]) == Zero ? 0 | 1;
    }
}
