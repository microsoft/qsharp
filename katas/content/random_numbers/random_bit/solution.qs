namespace Quantum.Kata.Reference {

    // Exercise 1.
    operation RandomBit () : Int {
        // Allocate single qubit
        use q = Qubit();
        
        // Set qubit in superposition state
        H(q);
        
        // Measuring state of qubit and return integer value of result
        return (M(q) == Zero) ? 0 | 1;
    }

}
