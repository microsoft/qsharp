namespace Kata {
    open Microsoft.Quantum.Math;

    function ComplexConjugate(x : Complex) : Complex {
        Complex(x.Real, -x.Imag)
    }
}
