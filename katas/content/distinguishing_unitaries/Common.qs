namespace Kata.Verification {
    import Std.Math.*;
    import Std.Random.*;

    // "Framework" operation for testing tasks for distinguishing unitaries
    // "unitaries" is the list of unitaries that can be passed to the task
    // "testImpl" - the solution to be tested
    // "unitaryNames" - the labels of unitaries in the list
    // "maxCalls" - max # of calls to the unitary that are allowed (-1 means unlimited) - currently unused, TODO use after #1154
    operation DistinguishUnitaries_Framework<'UInput>(
        unitaries : ('UInput => Unit is Adj + Ctl)[],
        testImpl : ('UInput => Unit is Adj + Ctl) => Int,
        unitaryNames : String[],
        maxCalls : Int
    ) : Bool {

        let nUnitaries = Length(unitaries);
        let nTotal = 100;
        mutable wrongClassifications = [0, size = nUnitaries * nUnitaries]; // [i * nU + j] number of times unitary i was classified as j
        mutable unknownClassifications = [0, size = nUnitaries];            // number of times unitary i was classified as something unknown

        for i in 1..nTotal {
            // get a random integer to define the unitary used
            let actualIndex = DrawRandomInt(0, nUnitaries - 1);

            // get the solution's answer and verify that it's a match
            let returnedIndex = testImpl(unitaries[actualIndex]);

            if returnedIndex != actualIndex {
                if returnedIndex < 0 or returnedIndex >= nUnitaries {
                    set unknownClassifications w/= actualIndex <- unknownClassifications[actualIndex] + 1;
                } else {
                    let index = actualIndex * nUnitaries + returnedIndex;
                    set wrongClassifications w/= index <- wrongClassifications[index] + 1;
                }
            }
        }

        mutable totalMisclassifications = 0;
        for i in 0..nUnitaries - 1 {
            for j in 0..nUnitaries - 1 {
                let misclassifiedIasJ = wrongClassifications[(i * nUnitaries) + j];
                if misclassifiedIasJ != 0 {
                    set totalMisclassifications += misclassifiedIasJ;
                    Message($"Misclassified {unitaryNames[i]} as {unitaryNames[j]} in {misclassifiedIasJ} test runs.");
                }
            }
            if unknownClassifications[i] != 0 {
                set totalMisclassifications += unknownClassifications[i];
                Message($"Failed to classify {unitaryNames[i]} in {unknownClassifications[i]} test runs.");
            }
        }
        // This check will tell the total number of failed classifications
        if totalMisclassifications != 0 {
            Message($"{totalMisclassifications} test runs out of {nTotal} returned incorrect state.");
            Message("Incorrect.");
            return false;
        }
        Message("Correct!");
        true
    }

    operation MinusZ(q : Qubit) : Unit is Adj + Ctl {
        within {
            X(q);
        } apply {
            Z(q);
        }
    }

    operation XZ(q : Qubit) : Unit is Adj + Ctl {
        Z(q);
        X(q);
    }

    operation MinusY(q : Qubit) : Unit is Adj + Ctl {
        within {
            X(q);
        } apply {
            Y(q);
        }
    }

    operation MinusXZ(q : Qubit) : Unit is Adj + Ctl {
        X(q);
        Z(q);
    }
}
