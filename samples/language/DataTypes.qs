/// # Sample
/// Data Types
///
/// # Description
/// Q# has a pragmatic and intuitive type system. All data types in Q# are immutable. The available
/// primitive data types are: `Unit`, `Int`, `BigInt`, `Double`, `Bool`, `String`, `Qubit`, `Result`, 
/// `Pauli`, and `Range`. In addition to these primitive types, Q# offers primitive aggregate types
/// as well, `Array` and `Tuple`; composite types (a.k.a. user-defined types or UDTs); and function
/// and operation types.
namespace MyQuantumApp {
    
    /// In the below code, all varibles have type annotations to showcase their type.
    @EntryPoint()
    operation MeasureOneQubit() : Unit {
        // Notably, Qubits are allocated with the `use` keyword instead of declared with the `let`
        // keyword.
        // The resulting value represents an opaque identifier by which virtual quantum memory
        // can be addressed. Values of type Qubit are instantiated via allocation.
        use q: Qubit = Qubit();  

        // A 64-bit signed integer.
        let integer: Int = 42;

        // The singleton type whose only value is ().
        let unit: Unit = ();

        // BigInt literals are always suffixed with an L, and can be declared in
        // binary, octal, decimal, or hexadecimal.
        let binaryBigInt: BigInt = 0b101010L;
        let octalBigInt = 0o52L;
        let decimalBigInt = 42L;
        let hexadecimalBigInt = 0x2aL;

        // A double-precision 64-bit floating-point number.
        let double = 42.0;

        // Boolean values. Possible values are `true` or `false`.
        let bool = true;

        // Text as values that consist of a sequence of UTF-16 code units.
        let string = "";

        // Represents the result of a projective measurement onto the eigenspaces
        // of a quantum operator with eigenvalues ±1. Possible values are `Zero` or `One`.
        let result = One;

        // A single-qubit Pauli matrix. Possible values are PauliI, PauliX, PauliY, or PauliZ.
        let pauli = [PauliX, PauliY, PauliZ];

        // Represents an ordered sequence of equally spaced Int values.
        // Values may represent sequences in ascending or descending order.
        let range = 1..100;

        // A collection that contains a sequence of values of the same type.
        let array_of_ints = [1, 2, 3];

        // A tuple contains a fixed number of items of potentially different types. 
        // Tuples containing a single element are equivalent to the element they contain.
        let tuple = (1, "one", One);

        // A user-defined-type (UDT) consisting of two named parameters, `Real` and `Imaginary`,
        // and one anonymous parameter of Boolean type.
        newtype ComplexBool = (Real: Double, Imaginary : Double, Bool);
        // Instantiation of the above UDT.
        let complex = ComplexBool(42.0, 0.0, false);

        // A function that takes an integer and returns a boolean. This variable declaration
        // uses a Lambda function as its right hand side.
        // The function signature is provided as an annotation here, for clarity.
        let functionType: Int => Bool = (int) => int == 0;
    }

}