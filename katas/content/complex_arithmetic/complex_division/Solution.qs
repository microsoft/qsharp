namespace Kata {    
    open Microsoft.Quantum.Math;
    
    operation ComplexDiv(x : Complex, y: Complex) : Complex {

    let (a, b) = x!;
    let (c, d) = y!;

    let denominator = c^2.0 + d^2.0;

    let real = ((a * c) + (b * d)) / denominator;
    let imag = ((a * ( - d)) + (b * c)) / denominator;

    return Complex(real, imag);
    }
}