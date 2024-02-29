namespace Kata {    
     open Microsoft.Quantum.Math;
     
    operation ComplexAdd(x : Complex, y: Complex) : Complex {
        
        let a = x::Real;
        let b = x::Imag;

        let c = y::Real;
        let d = y::Imag;
    
        let z = Complex(a + c, b + d);

        return z;
    }
}
