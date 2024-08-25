namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    function TensorProduct_Reference() : Double[][] {
        return [[5., 6., 10., 12.],
                [7., 8., 14., 16.],
                [15., 18., 20., 24.],
                [21., 24., 28., 32.]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 2;
        ArraysEqualD(Kata.TensorProduct(), TensorProduct_Reference(), precision)
    }
}
