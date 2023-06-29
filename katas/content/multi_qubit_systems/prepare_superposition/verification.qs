namespace Quantum.Kata.Reference {

    @EntryPoint()
    operation T2_PrepareState2 () : Unit {
        AssertEqualOnZeroState(PrepareState2, PrepareState2_Reference);
    }

}
