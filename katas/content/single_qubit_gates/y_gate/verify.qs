namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    operation ApplyYReference(q : Qubit) : Unit is Adj + Ctl {
        Y(q);
    }

    operation VerifyExercise() : Bool {
        let task = ApplyY;
        let taskRef = ApplyYReference;

        mutable isCorrect = false;

        // Explicit scopes are used to make output from DumpMachine calls more useful.
        {
            use (ctl, target) = (Qubit(), Qubit());
            within {
                H(ctl);
            }
            apply {
                Controlled task([ctl], target);
                Adjoint Controlled taskRef([ctl], target);
            }
            set isCorrect = CheckAllZero([ctl, target]);
            ResetAll([ctl, target]);
        }

        if isCorrect {
            use target = Qubit();
            task(target);
            Message("Qubit state after applying the Y gate to the |0⟩ state:");
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