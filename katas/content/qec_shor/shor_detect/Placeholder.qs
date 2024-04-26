namespace Kata {
    operation ShorDetectError (qs : Qubit[]) : (Int, Pauli) {
        // Implement your solution here...

        return (-2, PauliI);
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