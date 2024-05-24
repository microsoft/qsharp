namespace Kata {
    open Microsoft.Quantum.Math;    
    operation DistinguishRzFromR1 (unitary : ((Double, Qubit) => Unit is Adj+Ctl)) : Int {
        use qs = Qubit[2];
        within {
            H(qs[0]);
        } apply {
            Controlled unitary(qs[0..0], (2.0 * PI(), qs[1]));
        }
        return M(qs[0]) == Zero ? 1 | 0;
    }
}
