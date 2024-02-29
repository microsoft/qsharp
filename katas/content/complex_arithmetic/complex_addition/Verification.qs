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
        
        mutable success = false;

        let testx = ComplexRandom(0., 100.); 
        let testy = ComplexRandom(0., 100.);

        mutable expected = ComplexAdd_Reference(testx, testy); 
        mutable actual = Kata.ComplexAdd(testx, testy);        
        
        if (ComplexEqual(expected, actual)) {
            set success = true; 
            Message("Correct!");
        }        
        
        else {
            Message("Incorrect. Actual sum doesn't match expected value");
            Message($"Actual value: {actual::Real} + {actual::Imag}i. Expected value: {expected::Real} + {expected::Imag}i");
        }        
        
        return (success);
        
    }
    
}
