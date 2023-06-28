namespace Quantum.Kata.Reference {

    open Microsoft.Quantum.Math;

    // Exercise 4. 
    operation WeightedRandomBit (x : Double) : Int {
        // Calculate theta value
        let theta = 2.0 *  ArcCos(Sqrt(x));  // (or) 2.0 * ArcSin(Sqrt(1.0 - x));

        // Allocate single qubit
        use q = Qubit();

        // Set qubit in superposition state which aligns with given probabilities
        Ry(theta, q);

        // Measuring state of qubit and return integer value of result
        return M(q) == Zero ? 0 | 1;
    }

}
