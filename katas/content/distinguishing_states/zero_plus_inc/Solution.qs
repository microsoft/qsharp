namespace Kata {
    operation IsQubitZeroOrPlusOrInconclusive (q : Qubit) : Int {
        let basis = DrawRandomInt(0, 1);

        // randomize over std and had
        if basis == 0 {

            // use standard basis
            let result = M(q);
            // result is One only if the state was |+⟩
            return result == One ? 1 | -1;
        }
        else {
            // use Hadamard basis
            H(q);
            let result = M(q);
            // result is One only if the state was |0⟩
            return result == One ? 0 | -1;
        }
    }
}
