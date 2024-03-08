namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random; 
    
    operation ComplexModulus_Reference(x : Complex) : Double {
        
         return (Sqrt(AbsSquaredComplex(x)));
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        
        for count in 0 .. 24 {
            let testx = ComplexRandom(0., 100.); 
            
            let expected = ComplexModulus_Reference(testx); 
            let actual = Kata.ComplexModulus(testx);        
        
            if  expected != actual {
                Message($"Incorrect. When x = {testx::Real} + {testx::Imag}i, actual value doesn't match expected value");
                Message($"Actual value: {actual}. Expected value: {expected}");
                return false;
            }                
        }            

            Message("Correct!");
            return true;        
    }
}
