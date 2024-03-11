namespace Kata { 
    open Microsoft.Quantum.Math;

    function ComplexModulus(x : Complex) : Double {
        let (a, b) = x!;
        return Sqrt(a * a + b * b);
    }
}
