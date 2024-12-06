namespace Kata.Verification {
    import Std.Math.*;
    import KatasUtils.*;

    operation ReflectionAboutState(
        qs : Qubit[],
        statePrep : Qubit[] => Unit is Adj + Ctl
    ) : Unit is Adj + Ctl {
        within {
            Adjoint statePrep(qs);
        } apply {
            ConditionalPhaseFlip(qs);
        }
    }

    // You might find this helper operation from an earlier task useful.
    operation ConditionalPhaseFlip(qs : Qubit[]) : Unit is Adj + Ctl {
        within {
            ApplyToEachA(X, qs);
        } apply {
            Controlled Z(qs[1...], qs[0]);
        }
        R(PauliI, 2.0 * PI(), qs[0]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (N, statePrep) in [(2, qs => I(qs[0])), (3, ApplyToEachCA(H, _)), (1, qs => Ry(0.5, qs[0]))] {
            let sol = Kata.ReflectionAboutState(_, statePrep);
            let ref = ReflectionAboutState(_, statePrep);
            if not CheckOperationsAreEqualStrict(N, sol, ref) {
                Message("Incorrect.");
                return false;
            }
        }
        Message("Correct!");
        true
    }
}
