namespace Kata {
    operation Oracle_And(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        Controlled X(x, y);
    }        
}
