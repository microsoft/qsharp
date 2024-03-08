namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;    
    open Microsoft.Quantum.Convert; 

    operation ComplexAdd_Reference(x : Complex, y : Complex) : Complex {
    
        // Calculate the sum of two complex numbers.  
        let z = Complex (x::Real + y::Real, x::Imag + y::Imag);
        return z;
        }

    @EntryPoint()
    operation CheckSolution() : Bool {
        
        for count in 0 .. 24 {
            let testx = ComplexRandom(0., 100.); 
            let testy = ComplexRandom(0., 100.);

            let expected = ComplexAdd_Reference(testx, testy); 
            let actual = Kata.ComplexAdd(testx, testy);        
        
            if not(ComplexEqual(expected, actual)) {
                Message($"Incorrect. When x = {testx::Real} + {testx::Imag}i and y = {testy::Real} + {testy::Imag}i, actual value doesn't match expected value");
                Message($"Actual value: {actual::Real} + {actual::Imag}i. Expected value: {expected::Real} +  {expected::Imag}i");
                return false;
            }                
        }            

            Message("Correct!");
            return true;        
    }    
}
