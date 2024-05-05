namespace Kata {
    operation PhaseChange (alpha : Double, q : Qubit) : Unit is Adj + Ctl {
        R1(alpha, q); 
    }
}
