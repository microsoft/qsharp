namespace Kata {
    operation ReconstructAndMeasureMessage(qBob : Qubit, (b1 : Bool, b2 : Bool), basis : Pauli) : Result {
        ReconstructMessage(qBob, (b1, b2));
        return Measure([basis], [qBob]);
    }

    operation ReconstructMessage(qBob : Qubit, (b1 : Bool, b2 : Bool)) : Unit {
        if b1 {
            Z(qBob);
        }
        if b2 {
            X(qBob);
        }
    }
}