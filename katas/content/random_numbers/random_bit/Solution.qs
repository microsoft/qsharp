namespace Kata {
    operation RandomBit() : Int {
        // Allocate single qubit.
        use q = Qubit();

        // Convert the qubit state to |+>.
        H(q);

        // Measure the qubit and reset it to |0>.
        let result = MResetZ(q);

        // Return integer value of result.
        if result == One {
            return 1;
        }
        return 0;
    }
}
