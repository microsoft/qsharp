namespace Kata.Verification{
    operation StatePrep_BasisStateMeasurement(
        qs : Qubit[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        if state / 2 == 1 {
            // |10⟩ or |11⟩
            X(qs[0]);
        }

        if state % 2 == 1 {
            // |01⟩ or |11⟩
            X(qs[1]);
        }
    }
}
