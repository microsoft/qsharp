namespace Quantum.Kata.Reference {

    // ------------------------------------------------------
    @EntryPoint()
    operation T31_Or_Oracle () : Unit {
        AssertTwoOraclesAreEqual(1..10, Or_Oracle, Or_Oracle_Reference);
    }

}
