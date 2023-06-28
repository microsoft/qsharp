namespace Quantum.Kata.Reference {
    
    // Exercise 3.
    operation RandomNBits_Reference (N: Int) : Int {
        mutable result = 0;
        for i in 0..(N - 1) {
            set result = result * 2 + RandomBit_Reference();
        }
        return result;
    }

}
