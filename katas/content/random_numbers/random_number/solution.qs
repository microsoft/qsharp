namespace Kata {
    open Microsoft.Quantum.Math;

    // Exercise 1.
    operation RandomBit(): Int {
        // Allocate single qubit
        use q = Qubit();
        
        // Set qubit in superposition state
        H(q);
        
        // Measuring state of qubit
        let result = M(q);
        
        // Reset qubit and return integer value of result
        if result == One {
            X(q);
            return 1;
        }
        return 0;
    }
    
    // Exercise 3.
    operation RandomNBits(N: Int): Int {
        mutable result = 0;
        for i in 0..(N - 1) {
            set result = result * 2 + RandomBit();
        }
        return result;
    }

    
    // Exercise 5.
    operation RandomNumberInRange(min: Int, max: Int): Int {
        let nBits = BitSizeI(max - min);
        mutable output = 0; 
        repeat {
            set output = RandomNBits(nBits); 
        } until output <= max - min;
        return output + min;
    }

}
