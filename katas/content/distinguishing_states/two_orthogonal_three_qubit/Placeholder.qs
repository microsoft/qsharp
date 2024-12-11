namespace Kata {
    import Std.Convert.*;
    import Std.Math.*;
    import Std.Measurement.*;
    operation ThreeQubitMeasurement(qs : Qubit[]) : Int {
        // Implement your solution here...

        return -1;
    }

    // You might find this helper operation from the task
    // "W State on Arbitrary Number of Qubits" from the "Preparing Quantum States" kata useful.
    operation WState_Arbitrary(qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);

        if N == 1 {
            X(qs[0]);
        } else {
            let theta = ArcSin(1.0 / Sqrt(IntAsDouble(N)));
            Ry(2.0 * theta, qs[0]);
            X(qs[0]);
            Controlled WState_Arbitrary(qs[0..0], qs[1..N - 1]);
            X(qs[0]);
        }
    }
}
