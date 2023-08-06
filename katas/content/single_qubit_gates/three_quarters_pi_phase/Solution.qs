namespace Kata {
    operation ThreeQuartersPiPhase (q : Qubit) : Unit is Adj+Ctl {
        S(q);
        T(q);
    }
}
