namespace Kata {
    
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Intrinsic;

    @EntryPoint()
    operation ComplexAdd(x : Complex, y: Complex) : Complex {
        
        let z = Complex(x::Real + y::Real, x::Imag + y::Imag);

        return z;
    }
}
