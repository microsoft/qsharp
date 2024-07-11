namespace OperationEquivalence {
    open Microsoft.Quantum.Diagnostics;
    open SWAP;

    operation TestEquivalence(): Unit {
        let actual = qs => SWAP.ApplySWAP(qs);
        let expected = qs => SWAP.ApplySWAP(qs);
        Fact(CheckOperationsAreEqual(2, actual, expected)==true, "Actual and expected operation should be same");
    }
}
