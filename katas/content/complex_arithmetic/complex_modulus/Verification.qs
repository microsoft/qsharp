namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random; 
    
    operation ComplexModulus_Reference(x : Complex) : Double {
        
         return (Sqrt(AbsSquaredComplex(x)));
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        
        mutable success = false;
        mutable count = 0;
        mutable expected = 0.0;
        mutable actual = 0.0;

        repeat {

            let testx = ComplexRandom(0., 100.); 
            
            set expected = ComplexModulus_Reference(testx); 
            set actual = Kata.ComplexModulus(testx);        
        
            if expected == actual {
                set success = true; 
            }         

            set count += 1;        
        }
        until (count > 25) or (success == false);

        if success == true{Message("Correct!");}
        else {
              Message("Incorrect. Actual value doesn't match expected value");
              Message($"Actual value: {actual}  Expected value: {expected}");
            }

        return (success);
        
    }
}
