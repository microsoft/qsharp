namespace Kata {    
    open Microsoft.Quantum.Math;
    
    operation ComplexMult(x : Complex, y: Complex) : Complex {

        let a = x::Real;
        let b = x::Imag;

        let c = y::Real;
        let d = y::Imag;

        let real = (a * c) - (b * d);
        let imag = (a * d) + (b * c);
        
        let z = Complex(real, imag) ;

        return z;
    }
}
