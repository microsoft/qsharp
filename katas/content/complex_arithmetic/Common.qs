namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;    
    open Microsoft.Quantum.Convert;
    // open Microsoft.Quantum.Intrinsic;

    operation ComplexRandom(min : Double, max : Double) : Complex {
        
        // Generates a random complex number. 
            let real = DrawRandomDouble(min, max);
            let imag = DrawRandomDouble(min, max);
            return Complex (real, imag);
        }

    operation ComplexEqual(x : Complex, y : Complex) : Bool { 

        // Tests two complex numbers for equality.
            return ((x::Real == y::Real) and (x::Imag == y::Imag));
        }    

}
