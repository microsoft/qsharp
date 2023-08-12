namespace Kata {
    operation MultiControls (controls : Qubit[], target : Qubit, controlBits : Bool[]) : Unit is Adj + Ctl {
        for index in 0 .. Length(controls) - 1 {
            if controlBits[index] == false {
                X(controls[index]);       
            }
        }
     
        Controlled X(controls,target);
    
        for index in 0 .. Length(controls) - 1 {
            if controlBits[index] == false {
                X(controls[index]);       
            }
        }
    }
}