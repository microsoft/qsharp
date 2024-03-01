namespace Kata {    
    open Microsoft.Quantum.Math;
    
    operation ComplexDiv(x : Complex, y: Complex) : Complex {

        let a = x::Real;
        let b = x::Imag;

        let c = y::Real;
        let d = y::Imag;
   
        let denominator = ((c * c) + (d * d));

        let real = ((a * c) + (b * d)) / denominator;
        let imag = ((a * ( - d)) + (b * c)) / denominator;
        
        let z = Complex(real, imag);

        return z;
    }
}