namespace Quantum.Kata.Reference {

    @EntryPoint()
    operation T3_PrepareState3 () : Unit {
        AssertEqualOnZeroState(PrepareState3, PrepareState3_Reference);
    }

}
