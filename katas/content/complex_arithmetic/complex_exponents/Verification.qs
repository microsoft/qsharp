namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;    
    
    @EntryPoint()
    operation CheckSolution() : Bool {
        
        for count in 0 .. 24 {
            let testx = ComplexRandom(0., 100.); 
            
            let expected = ComplexExponent_Reference(testx); 
            let actual = Kata.ComplexExponent(testx);        
        
            if not(ComplexEqual(expected, actual)) {
                Message($"Incorrect. When x = {testx::Real} + {testx::Imag}i, actual value doesn't match expected value");
                Message($"Actual value: {actual::Real} + {actual::Imag}i. Expected value: {expected::Real} +  {expected::Imag}i");
                return false;
            }                
        }            

            Message("Correct!");
            return true;        
    }
    
}

