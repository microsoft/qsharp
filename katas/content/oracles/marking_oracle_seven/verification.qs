namespace Quantum.Kata.Reference {

    // ------------------------------------------------------
    @EntryPoint()
    operation T13_IsSeven_MarkingOracle () : Unit {
        AssertTwoOraclesAreEqual(3..3, IsSeven_MarkingOracle, IsSeven_MarkingOracle_Reference);
    }

}
