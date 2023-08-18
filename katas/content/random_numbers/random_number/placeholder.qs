namespace Kata {
    operation RandomNumberInRange(min : Int, max : Int) : Int {
        // Implement your solution here...

        return -1;
    }

    // You can use this operation to implement your solution.
    operation RandomBit() : Int {
        // Allocate single qubit.
        use q = Qubit();

        // Set qubit in superposition state.
        H(q);

        // Measuring the qubit and reset.
        let result = M(q);
        Reset(q);

        // Return integer value of result.
        if result == One {
            return 1;
        }
        return 0;
    }
}
