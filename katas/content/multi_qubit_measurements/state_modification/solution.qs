namespace Kata.Reference {

    operation StateSelction(qs : Qubit[], ind : Int) : Unit {
        // It is convenient to convert measurement outcome to an integer
        let outcome = M(qs[0]) == Zero ? 0 | 1;
        if outcome != ind {
            X(qs[1]);
        }
    }

}
