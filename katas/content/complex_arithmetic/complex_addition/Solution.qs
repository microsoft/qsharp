namespace Kata {
    open Microsoft.Quantum.Math;

    function ComplexAdd(x : Complex, y : Complex) : Complex {
        Complex(x.Real + y.Real, x.Imag + y.Imag)
    }
}
