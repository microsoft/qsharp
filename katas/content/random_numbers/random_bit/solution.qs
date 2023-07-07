namespace Kata.Reference {

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

}
