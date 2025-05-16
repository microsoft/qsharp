namespace Kata {
    import Std.Random.*;

    operation RandomArray(N : Int) : Bool[] {
        mutable array = [false, size = N];
        for i in 0..N - 1 {
            array[i] = DrawRandomInt(0, 1) == 0;
        }
        return array;
    }
}
