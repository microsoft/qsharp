namespace Kata {
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation DemoBasisMeasurement() : Unit {
        let expected_probabilities = [1. / 9., 4. / 9., 0., 4. / 9.];

        // Set up counter array for tracking measurement outcomes.
        mutable countArray = [0, 0, 0, 0];

        use qs = Qubit[2];
        let numRuns = 1000;
        for i in 1..numRuns {
            // Prepare the starting state.
            Ry(2. * ArcCos(1. / 3.), qs[1]);
            Controlled H([qs[1]], qs[0]);
            if i == 1 {
                Message("The state of the system before measurement is:");
                DumpMachine();
            }

            // Measure the first (most significant) qubit, then measure the second (least significant) qubit,
            // and convert the result to an integer, interpreting it as big endian binary notation.
            let result = (MResetZ(qs[0]) == One ? 1 | 0) * 2 + (MResetZ(qs[1]) == One ? 1 | 0);

            set countArray w/= result <- countArray[result] + 1;
        }

        // Obtain simulated probability of measurement for each outcome.
        mutable simulated_probabilities = [];
        for i in 0..3 {
            set simulated_probabilities += [IntAsDouble(countArray[i]) / IntAsDouble(numRuns)];
        }

        Message($"Theoretical measurement probabilities are {expected_probabilities}");
        Message($"Simulated measurement probabilities are {simulated_probabilities}");
    }
}
