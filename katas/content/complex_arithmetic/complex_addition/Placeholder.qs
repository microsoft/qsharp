namespace Kata {    
    open Microsoft.Quantum.Math;
    
    function ComplexAdd(x : Complex, y : Complex) : Complex {    
        // Extract real and imaginary components of the inputs.
        let (a, b) = x!;
        let (c, d) = (y::Real, y::Imag);
        // Implement your solution here...
        return Complex(0., 0.);
    }
}
