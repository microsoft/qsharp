namespace Kata {
    import Std.Diagnostics.CheckZero;
    operation IsEigenvector(U : Qubit => Unit, P : Qubit => Unit is Adj) : Bool {
        use q = Qubit();
        P(q);
        U(q);
        Adjoint P(q);
        let ret = CheckZero(q);
        Reset(q);
        return ret;
    }
}
