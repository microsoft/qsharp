namespace Kata {
    open Microsoft.Quantum.Math;

    function ComplexDiv(x : Complex, y : Complex) : Complex {
        let (a, b) = (x.Real, x.Imag);
        let (c, d) = (y.Real, x.Imag);
        let denominator = c * c + d * d;
        let real = (a * c + b * d) / denominator;
        let imag = (- a * d + b * c) / denominator;
        return Complex(real, imag);
    }
}
