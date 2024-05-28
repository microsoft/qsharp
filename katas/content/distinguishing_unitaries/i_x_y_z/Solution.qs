namespace Kata {
    operation DistinguishPaulis (unitary : (Qubit => Unit is Adj + Ctl)) : Int {
        // apply operation to the 1st qubit of a Bell state and measure in Bell basis
        use qs = Qubit[2];
        within {
            H(qs[0]);
            CNOT(qs[0], qs[1]);
        } apply {
            unitary(qs[0]);
        }

        // after this I -> 00, X -> 01, Y -> 11, Z -> 10
        let ind = MeasureInteger(qs);
        let returnValues = [0, 3, 1, 2];
        return returnValues[ind];
    }
}
