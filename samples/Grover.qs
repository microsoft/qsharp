// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Samples.SimpleGrover {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Diagnostics;

    /// # Summary
    /// Implements Grover's algorithm which searches all possible inputs
    /// to an operation to find a particular marked state.
    operation GroverSearch(
        nQubits: Int,
        nIterations: Int,
        phaseOracle: Qubit[] => Unit): Result[] {

        use qubits = Qubit[nQubits];

        // Initialize a uniform superposition over all possible inputs.
        PrepareUniform(qubits);

        // The search itself consists of repeatedly reflecting about the
        // marked state and our start state, which we can write out
        // in Q# as a for loop.
        for idxIteration in 1..nIterations {
            phaseOracle(qubits);
            ReflectAboutUniform(qubits);
        }

        // Measure and return the answer.
        mutable results = [];
        for q in qubits {
            let result = M(q);
            set results = results + [result];
            if (result == One) {
                X(q);
            }
        }

        return results;
    }

    /// # Summary
    /// Given a register in the all-zeros state, prepares a uniform
    /// superposition over all basis states.
    operation PrepareUniform(inputQubits : Qubit[]): Unit is Adj + Ctl {
        for q in inputQubits {
            H(q);
        }
    }

    /// # Summary
    /// Reflects about the all-ones state.
    operation ReflectAboutAllOnes(inputQubits : Qubit[]): Unit {
        Controlled Z(Most(inputQubits), Tail(inputQubits));
    }

    /// # Summary
    /// Reflects about the uniform superposition state.
    operation ReflectAboutUniform(inputQubits : Qubit[]): Unit {
        within {
            // Transform the uniform superposition to all-zero.
            Adjoint PrepareUniform(inputQubits);
            // Transform the all-zero state to all-ones
            for q in inputQubits {
                X(q);
            }
        } apply {
            // Now that we've transformed the uniform superposition to
            // the all-ones state, reflect about the all-ones state,
            // then let the within/apply block transform us back.
            ReflectAboutAllOnes(inputQubits);
        }
    }


    /// # Summary
    /// Reflects about the basis state marked by alternating
    /// zeros and ones. This operation defines what input we
    /// are trying to find in the search.
    operation ReflectAboutMarked(inputQubits : Qubit[]) : Unit {
        Message("Reflecting about marked state...");
        use outputQubit = Qubit();
        within {
            // We initialize the outputQubit to (|0⟩ - |1⟩) / √2,
            // so that toggling it results in a (-1) phase.
            X(outputQubit);
            H(outputQubit);
            // Flip the outputQubit for marked states.
            // Here, we get the state with alternating 0s and 1s
            // by using the X instruction on every other qubit.
            for q in inputQubits[...2...] {
                X(q);
            }
        } apply {
            Controlled X(inputQubits, outputQubit);
        }
    }

    /// # Summary
    /// Returns the number of Grover iterations needed to find
    /// a single marked item, given the number of qubits in a register.
    function NIterations(nQubits : Int) : Int {
        let nItems = 1 <<< nQubits; // 2^numQubits
        // compute number of iterations:
        let angle = ArcSin(1. / Sqrt(IntAsDouble(nItems)));
        let nIterations = Round(0.25 * PI() / angle - 0.5);
        return nIterations;
    }

    /// # Summary
    /// Apply Grover's algorithm to find a particular marked state.
    @EntryPoint()
    operation Main() : Result[] {
        let nQubits = 6;

        // We are setting the number of iterations to 1 (instead of the
        // proper NIterations) to make the search less precise.
        // Then we use many shots to see if the probability of the
        // correct solution will still be high.
        // Use NIterations instead of 1 to get correct answer with
        // fewer shots.
        let nIterations = 1; // NIterations(nQubits);
        Message("Iterations: " + AsString(nIterations));

        let results =  GroverSearch(
            nQubits,
            nIterations,
            ReflectAboutMarked);
        Message("Done.");

        return results;
    }
}
