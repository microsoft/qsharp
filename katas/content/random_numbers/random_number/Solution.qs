namespace Kata {
    import Std.Math.*;

    operation RandomNumberInRange(min : Int, max : Int) : Int {
        let nBits = BitSizeI(max - min);
        mutable output = 0;
        repeat {
            set output = RandomNBits(nBits);
        } until output <= max - min;
        return output + min;
    }

    operation RandomNBits(N : Int) : Int {
        mutable result = 0;
        for i in 0..N - 1 {
            set result = result * 2 + RandomBit();
        }
        return result;
    }

    operation RandomBit() : Int {
        use q = Qubit();
        H(q);
        return MResetZ(q) == Zero ? 0 | 1;
    }
}
