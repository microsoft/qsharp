namespace Kata.Verification {

    // Exercise 3: Distinguish four basis states
    operation StatePrep_BasisStateMeasurement(qs : Qubit[], state : Int, dummyVar : Double) : Unit is Adj {
        if state / 2 == 1 {
            // |10⟩ or |11⟩
            X(qs[0]);
        }
        if state % 2 == 1 {
            // |01⟩ or |11⟩
            X(qs[1]);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        return DistinguishStates_MultiQubit(
            2,
            4,
            StatePrep_BasisStateMeasurement,
            Kata.BasisStateMeasurement,
            false,
            ["|00⟩", "|01⟩", "|10⟩", "|11⟩"]);
    }
}
