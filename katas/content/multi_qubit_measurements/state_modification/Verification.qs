namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;

    // State selection using partial measurements
    operation StateInitialize_StateSelection(alpha : Double, qs : Qubit[]) : Unit {
        // Prepare the state to be input to the testImplementation
        // set the second qubit in a superposition a |0⟩ + b|1⟩
        // with a = cos alpha, b = sin alpha
        Ry(2.0 * alpha, qs[1]);

        H(qs[0]);
        // Apply CX gate
        CX(qs[0], qs[1]);
    }

    // Prepare the expected state of the second qubit for the exercise.
    operation StatePrepare_StateSelection(alpha : Double, ind : Int, q : Qubit) : Unit is Adj {
        // set the second qubit in a superposition a|0⟩ + b|1⟩
        // with a = cos alpha, b = sin alpha
        Ry(2.0 * alpha, q);
        if ind == 1 {
            // change the state to b|0⟩ + a|1⟩
            X(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        use qs = Qubit[2];
        for i in 0..5 {
            let alpha = (PI() * IntAsDouble(i)) / 5.0;
            let params = $"a = {Cos(alpha)}, b = {Sin(alpha)}";

            for ind in 0..1 {
                // Prepare the state to be input to the testImplementation
                StateInitialize_StateSelection(alpha, qs);

                Kata.StateSelection(qs, ind);

                // Apply adjoint of state preparation operation
                Adjoint StatePrepare_StateSelection(alpha, ind, qs[1]);

                // We only care about the state of the second qubit;
                // if it's still entangled with the first one or not in zero state, this check will fail.
                if not CheckZero(qs[1]) {
                    ResetAll(qs);
                    Message("Incorrect.");
                    Message($"The state of the second qubit for {params}, ind = {ind} does not match expectation.");
                    return false;
                }

                Reset(qs[0]);
            }
        }
        Message("Correct!");
        true
    }

}
