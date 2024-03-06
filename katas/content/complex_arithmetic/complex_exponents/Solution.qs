namespace Kata { 
    open Microsoft.Quantum.Math;

    operation ComplexExponent (x : Complex) : Complex {
        
        let e = 2.7182818284590452354;
        
        let a = x::Real;
        let b = x::Imag;

        let g = e^a * Cos(b);
        let h = e^a * Sin(b);

        return Complex(g, h);
    }
}
