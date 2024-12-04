namespace Kata {
    import Std.Math.*;

    operation ReflectionAboutState(
        qs : Qubit[],
        statePrep : Qubit[] => Unit is Adj + Ctl
    ) : Unit is Adj + Ctl {
        // Implement your solution here...

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
}
