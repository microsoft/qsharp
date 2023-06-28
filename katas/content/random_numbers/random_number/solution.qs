namespace Quantum.Kata.Reference {
    
    // Exercise 5.
    operation RandomNumberInRange_Reference (min : Int, max : Int) : Int {
        let nBits = BitSizeI(max - min);
        mutable output = 0; 
        repeat {
            set output = RandomNBits_Reference(nBits); 
        } until output <= max - min;
        return output + min;
    }

}
