namespace Kata {
    operation MultiControls (controls : Qubit[], target : Qubit, controlBits : Bool[]) : Unit is Adj + Ctl {
        within {
            for index in 0 .. Length(controls) - 1 {
                if controlBits[index] == false {
                    X(controls[index]);       
                }
            }
        } apply {
            Controlled X(controls,target);
        }
    }
}