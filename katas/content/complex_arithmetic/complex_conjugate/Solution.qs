namespace Kata {    
    open Microsoft.Quantum.Math;    

    operation ComplexConjugate(x : Complex) : Complex {
        return Complex(x::Real, -x::Imag);
    }
}
