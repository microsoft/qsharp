namespace Kata { 
    open Microsoft.Quantum.Math;

    operation ComplexExponent (x : Complex) : Complex {
        
        let (a, b) = x!;
        
        let g = E()^a * Cos(b);
        let h = E()^a * Sin(b);

        return Complex(g, h);
    }
}
