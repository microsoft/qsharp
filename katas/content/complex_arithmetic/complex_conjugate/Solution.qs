namespace Kata {
    open Microsoft.Quantum.Math;

    operation ComplexConjugate(x : Complex) : Complex {
        Complex(x.Real, -x.Imag)
    }
}
