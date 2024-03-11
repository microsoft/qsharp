namespace Kata { 
    open Microsoft.Quantum.Math;

    function ComplexExponent (x : Complex) : Complex {
        let (a, b) = x!;
        return Complex(E()^a * Cos(b), E()^a * Sin(b));
    }
}
