namespace Quantum.Kata.Reference {

    @EntryPoint()
    operation T1_PrepareState1 () : Unit {
        AssertEqualOnZeroState(PrepareState1, PrepareState1_Reference);
    }

}
