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
        
        let ce = ((AbsD(x::Real - y::Real) <= 0.001) and (AbsD(x::Imag - y::Imag) <= 0.001)); 
        return(ce); 

        // Tests two complex numbers for equality.
         //   return ((x::Real == y::Real) and (x::Imag == y::Imag));

        }


         operation ComplexMult_Reference(x : Complex, y : Complex) : Complex {
    
        // Calculate the product of two complex numbers.  
            let z = Complex ( x::Real * y::Real - x::Imag * y::Imag, x::Real * y::Imag + x::Imag * y::Real);
            return z;

        }

        operation ComplexConjugate_Reference(x : Complex) : Complex {
    
        // Return the complex conjugate  
            let z = Complex ( x::Real, - x::Imag);
            return z;

        }    

}
