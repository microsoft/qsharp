namespace Kata.Verification {
    import Std.Math.*;

    function TensorProduct_Reference() : Double[][] {
        return [
            [5., 6., 10., 12.],
            [7., 8., 14., 16.],
            [15., 18., 20., 24.],
            [21., 24., 28., 32.]
        ];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualD(Kata.TensorProduct(), TensorProduct_Reference())
    }
}
