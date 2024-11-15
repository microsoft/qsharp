namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Math.*;
    import KatasUtils.*;

    // Measure state in {|A❭, |B❭} basis
    // |A⟩ =   cos(alpha) * |0⟩ - i sin(alpha) * |1⟩,
    // |B⟩ = - i sin(alpha) * |0⟩ + cos(alpha) * |1⟩.
    operation StatePrep_IsQubitA(alpha : Double, q : Qubit, state : Int) : Unit is Adj {
        if state == 0 {
            // convert |0⟩ to |B⟩
            X(q);
            Rx(2.0 * alpha, q);
        } else {
            // convert |0⟩ to |A⟩
            Rx(2.0 * alpha, q);
        }
    }

    // We can use the StatePrep_IsQubitA operation for the testing
    operation CheckSolution() : Bool {
        for i in 0..10 {
            let alpha = (PI() * IntAsDouble(i)) / 10.0;
            let isCorrect = DistinguishTwoStates_SingleQubit(
                StatePrep_IsQubitA(alpha, _, _),
                q => Kata.MeasureInABBasis(alpha, q) == Zero,
                [$"|B⟩=(-i sin({i}π/10)|0⟩ + cos({i}π/10)|1⟩)", $"|A⟩=(cos({i}π/10)|0⟩ + i sin({i}π/10)|1⟩)"],
                true
            );
            if not isCorrect {
                let precision = 3;
                Message($"Test fails for alpha={DoubleAsStringWithPrecision(alpha, precision)}");
                return false;
            }
        }
        Message("Correct!");
        true
    }

}
