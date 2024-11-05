namespace Kata {   
    operation AllBellStates (qs : Qubit[], index : Int) : Unit is Adj + Ctl {
        H(qs[0]);
        
        if index == 1 {
            Z(qs[0]);
        }
        if index == 2 {
            X(qs[1]);
        }
        if index == 3 {
            Z(qs[0]);
            X(qs[1]);
        }
        
        CNOT(qs[0], qs[1]);
    }
}
