namespace Kata {    
    open Microsoft.Quantum.Math;    

    operation ComplexConjugate(x : Complex) : Complex {

        let real = x::Real;
        let imag  = - x::Imag;     
        let z = Complex(real, imag);

        return z;
    }
}
