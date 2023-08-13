namespace Kata {
    open Microsoft.Quantum.Canon;

    operation MultiControls (controls : Qubit[], target : Qubit, controlBits : Bool[]) : Unit is Adj + Ctl {
        ApplyControlledOnBitString(controlBits, X, controls, target);
    }
}