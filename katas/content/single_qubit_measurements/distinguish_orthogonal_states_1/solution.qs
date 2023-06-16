namespace Kata.Reference {
    open Microsoft.Quantum.Math;

    // Exercise 5.
    operation IsQubitPsiPlus (q : Qubit) : Bool { 
        Ry(-2.0 * ArcTan2(0.8, 0.6), q);
        return M(q) == Zero;
    }

}
