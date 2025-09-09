// # Sample
// Getting started
//
// # Description
// This is a minimal Q# program that can be used to start writing Q# code.

operation Main() : Unit {
    // TODO: Write your Q# code here.
    use q = Qubit();
    for i in 1..10 {
        H(q);
    }
        let result = M(q);
        Message($"Measurement result: {result}");
        Reset(q);
}
