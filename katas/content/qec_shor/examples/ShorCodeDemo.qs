namespace Kata {
    import Std.Arrays.*;
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation RunExample() : Unit {
        // Allocate qubits and prepare initial logical state.
        use qs = Qubit[9];
        Ry(ArcCos(0.6), qs[0]);

        // Encode the logical state using Shor code.
        ShorEncode(qs);

        // Introduce an arbitrary error on one qubit.
        Rz(0.1, qs[5]);

        // Detect and correct error.
        let (ind, err) = ShorDetectError(qs);

        if ind > -1 {
            if err == PauliX {
                X(qs[ind]);
            } elif err == PauliY {
                Y(qs[ind]);
            } else {
                ForEach(q => Z(q), qs[ind * 3..ind * 3 + 2]);
            }
        }

        // Check that the state was error-corrected accurately by uncomputing
        // the encoding and state preparation to see that the result is the |0⟩ state.
        Adjoint ShorEncode(qs);
        Ry(-ArcCos(0.6), qs[0]);
        if not CheckAllZero(qs) {
            Message("State recovered incorrectly!");
            ResetAll(qs);
        } else {
            Message("State recovered correctly!");
        }
    }

    operation ShorEncode(qs : Qubit[]) : Unit is Adj + Ctl {
        BitflipEncode(qs[0..3..8]);
        ApplyToEachCA(H, qs[0..3..8]);
        for i in 0..2 {
            BitflipEncode(qs[3 * i..3 * i + 2]);
        }
    }

    operation BitflipEncode(qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
    }

    operation ShorDetectError(qs : Qubit[]) : (Int, Pauli) {
        // Detect X error first
        mutable x_ind = -1;
        for i in 0..2 {
            let err_ind = BitflipDetectError(qs[3 * i..3 * i + 2]);
            if err_ind > -1 {
                set x_ind = 3 * i + err_ind;
            }
        }

        // Detect Z error
        mutable z_ind = -1;
        let m1 = Measure([PauliX, size = 6], qs[0..5]);
        let m2 = Measure([PauliX, size = 6], qs[3..8]);

        if m1 == Zero and m2 == Zero {
            set z_ind = -1;
        } elif m1 == One and m2 == Zero {
            set z_ind = 0;
        } elif m1 == One and m2 == One {
            set z_ind = 1;
        } elif m1 == Zero and m2 == One {
            set z_ind = 2;
        }

        // Combine both errors into return value
        if x_ind == -1 and z_ind == -1 {
            return (-1, PauliI);
        }
        if x_ind > -1 and z_ind > -1 {
            return (x_ind, PauliY);
        }
        if x_ind > -1 {
            return (x_ind, PauliX);
        }
        return (z_ind, PauliZ);
    }

    operation BitflipDetectError(qs : Qubit[]) : Int {
        let m1 = Measure([PauliZ, PauliZ], qs[0..1]);
        let m2 = Measure([PauliZ, PauliZ], qs[1..2]);

        if m1 == One and m2 == Zero {
            return 0;
        } elif m1 == One and m2 == One {
            return 1;
        } elif m1 == Zero and m2 == One {
            return 2;
        } else {
            return -1;
        }
    }
}
