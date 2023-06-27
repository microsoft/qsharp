namespace Quantum.Kata.Reference {

    // Task 4.1.
    operation ArbitraryBitPattern_Oracle_Reference (x : Qubit[], y : Qubit, pattern : Bool[]) : Unit  is Adj + Ctl {
        let PatternOracle = ControlledOnBitString(pattern, X);
        PatternOracle(x, y);
    }

}
