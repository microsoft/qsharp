namespace Kata {
    operation ShorDetectError (qs : Qubit[]) : (Int, Pauli) {
        // Detect X error first
        mutable x_ind = -1;
        for i in 0 .. 2 {
            let err_ind = BitflipDetectError(qs[3 * i .. 3 * i + 2]);
            if err_ind > -1 {
                set x_ind = 3 * i + err_ind;
            }
        }

        // Detect Z error 
        mutable z_ind = -1;
        let m1 = Measure([PauliX, size = 6], qs[0 .. 5]);
        let m2 = Measure([PauliX, size = 6], qs[3 .. 8]);

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

    // You might find this helper operation from an earlier task useful.
    operation BitflipDetectError (qs : Qubit[]) : Int {
        let m1 = Measure([PauliZ, PauliZ], qs[0 .. 1]);
        let m2 = Measure([PauliZ, PauliZ], qs[1 .. 2]);
        
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