namespace Kata {
    operation PhaseflipDetectError (qs : Qubit[]) : Int {
        let m1 = Measure([PauliX, PauliX], qs[0 .. 1]);
        let m2 = Measure([PauliX, PauliX], qs[1 .. 2]);
        
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