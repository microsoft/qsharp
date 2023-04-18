namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    operation ApplyYReference(q : Qubit) : Unit is Adj + Ctl {
        body ... {
            Y(q);
        }
        adjoint self;
    }

    operation Verify() : Bool {
        let task = ApplyY;
        let taskRef = ApplyYReference;

        use (aux, target) = (Qubit(), Qubit());
        H(aux);
        CNOT(aux, target);

        task(target);
        Adjoint taskRef(target);

        CNOT(aux, target);
        H(aux);

        if CheckAllZero([aux, target]) {
            task(target);
            DumpMachine();
            return true;
        }

        ResetAll([aux, target]);

        // Use DumpMachine to display actual vs desired state.
        task(target);
        DumpMachine();
        Reset(target);
        taskRef(target);
        DumpMachine();

        return false;
    }
}