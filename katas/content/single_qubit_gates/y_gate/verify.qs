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
            Message("Qubit state after applying the Y gate to the |0⟩ state:");
            task(target);
            DumpMachine();
            Reset(target);
        } else {
            {
                use expected = Qubit();
                Message("Expected state after applying operation:");
                taskRef(expected);
                DumpMachine();
                Reset(expected);
            }

            {
                use actual = Qubit();
                Message("Actual state after applying operation:");
                task(actual);
                DumpMachine();
                Reset(actual);
            }
        }

        return isCorrect;
    }
}