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

        mutable isCorrect = false;

        // Explicit scopes are used to make output from DumpMachine calls more useful.
        {
            use (ctl, target) = (Qubit(), Qubit());
            H(ctl);
            Controlled task([ctl], target);
            Adjoint Controlled taskRef([ctl], target);
            H(ctl);
            set isCorrect = CheckAllZero([ctl, target]);
            ResetAll([ctl, target]);
        }

        if isCorrect {
            use target = Qubit();
            task(target);
            Message("Qubit state after applying a global phase to the |0⟩ state:");
            DumpMachine();
            Reset(target);
        } else {
            {
                use expected = Qubit();
                taskRef(expected);
                Message("Expected state after applying operation:");
                DumpMachine();
                Reset(expected);
            }

            {
                use actual = Qubit();
                task(actual);
                Message("Actual state after applying operation:");
                DumpMachine();
                Reset(actual);
            }
        }

        return isCorrect;
    }
}