namespace Quantum.Kata.Reference {

    @EntryPoint()
    operation T4_PrepareState4 () : Unit {
        AssertEqualOnZeroState(PrepareState4, PrepareState4_Reference);
    }

}
