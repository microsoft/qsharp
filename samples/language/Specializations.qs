/// # Sample
/// Specializations
///
/// # Description
/// Q# allows specialized implementations. Operations in Q# can implicitly
/// or explicitly define adjoint and/or controlled versions.
/// Q# employs symbolic computation that can automatically generate the
/// corresponding adjoint and controlled implementations for a particular 
/// body implementation.
namespace MyQuantumApp {

    /// Implicit
    operation DoNothing() : Unit 
        is Adj + Ctl { }

    /// Explicit
    operation SWAP (q1 : Qubit, q2 : Qubit) : Unit
        is Adj + Ctl { 

        body (...) {
            CNOT(q1, q2);
            CNOT(q2, q1);
            CNOT(q1, q2);
        }

        adjoint (...) { 
            SWAP(q1, q2);
        }

        controlled (cs, ...) { 
            CNOT(q1, q2);
            Controlled CNOT(cs, (q2, q1));
            CNOT(q1, q2);            
        } 
    }
}