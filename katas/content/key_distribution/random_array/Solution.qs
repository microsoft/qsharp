namespace Kata {
    import Std.Random.*;

    operation RandomArray(N : Int) : Bool[] {
        mutable array = [false, size = N];
        for i in 0..N - 1 {
            set array w/= i <- DrawRandomInt(0, 1) == 0;
        }
        return array;
    }
}
