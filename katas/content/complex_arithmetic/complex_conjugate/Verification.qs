    namespace Kata.Verification {
        
        open Microsoft.Quantum.Math;
        open Microsoft.Quantum.Random;    
        open Microsoft.Quantum.Convert; 

        operation ComplexConjugate_Reference(x : Complex) : Complex {
    
        // Calculate the product of two complex numbers.  
            let z = Complex ( x::Real, - x::Imag);
            return z;

        }

        operation ComplexRandom(min : Double, max : Double) : Complex{
        
        // Generates a random complex number. 
            let real = DrawRandomDouble(min, max);
            let imag = DrawRandomDouble(min, max);
            return Complex (real, imag);
        }

        operation ComplexEqual(x : Complex, y : Complex) : Bool{

        // Tests two complex numbers for equality.
            return ((x::Real == y::Real) and (x::Imag == y::Imag));
        }
    
        @EntryPoint()
        operation CheckSolution() : Bool {
        
            mutable success = false;

            let testx = ComplexRandom(0., 100.); 
            
            mutable expected = ComplexConjugate_Reference(testx); 
            mutable actual = Kata.ComplexConjugate(testx);        
        
            if (ComplexEqual(expected, actual)) {
                set success = true; 
                Message("Correct!");
            }        
        
            else {
            Message("Incorrect. Actual value doesn't match expected value");
            Message($"Actual value: {actual::Real} + {actual::Imag}i. Expected value: {expected::Real} + {expected::Imag}i");
            }        
        
            return (success);
        
        }
    
    }