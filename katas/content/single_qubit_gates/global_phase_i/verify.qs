namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    operation GlobalPhaseIReference(q : Qubit) : Unit is Adj + Ctl {
        body ... {
            X(q);
            Z(q);
            Y(q);
        }
        adjoint ... {
            Y(q);
            Z(q);
            X(q);
        }
    }

    operation Verify() : Bool {
        let task = GlobalPhaseI;
        let taskRef = GlobalPhaseIReference;

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