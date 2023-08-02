namespace Kata {
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

    operation RandomNBits(N: Int): Int {
        mutable result = 0;
        for i in 0..(N - 1) {
            set result = result * 2 + RandomBit();
        }
        return result;
    }

}
