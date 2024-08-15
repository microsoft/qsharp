// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/*
Narrative: Want to show code, math, job results, and histograms in a single conversation
- Show me the code for a QRNG?
- Run that with a 5% depolarizing noise model
- How can I submit this to Azure using Python?
- Show me the last 5 jobs I ran (results in table)
*/

export default {
  code: `The below Q# code will generate a random number from 0 to 7:

\`\`\`qsharp
operation Main() : Result[] {
    // Generate 3-bit random number.
    let nBits = 3;
    return GenerateNRandomBits(nBits);
}

operation GenerateNRandomBits(nBits : Int) : Result[] {
    use register = Qubit[nBits];

    // Set the qubits into superposition of 0 and 1 using the Hadamard operation 'H'.
    for qubit in register {
        H(qubit);
    }

    let results = MResetEachZ(register);
    return results;
}
\`\`\`

This code is "base profile" compliant, meaning it should run on any quantum hardware.
`,
  noise: `
Depolarizing noise is defined for a single qubit as:

$$\\mathcal{E}(\\rho) = (1 - p)\\rho + \\frac{p}{3}(X\rho X + Y\\rho Y + Z\\rho Z)$$

When run with a 5% value for $p$ and applied equally to all single-qubit operators, the resulting histogram was:

\`\`\`widget
Histogram
\`\`\`
`,
  azure: `Use the below Python code to submit the above Q# to Azure Quantum:

\`\`\`python
import qsharp

def submit_to_azure():
    qsharp.compile("Main.qs")
    qsharp.submit(AZURE_QUANTUM_WORKSPACE, "my_job_name")

submit_to_azure()
\`\`\`

This requires the AZURE_QUANTUM_WORKSPACE to be set in your environment to your workspace credientials.
`,
  jobs: `Below shows the last 5 jobs run against the currently configured workspace:

\`\`\`widget
Results
\`\`\`
`,
};

export function* mock_stream(data: string): Generator<string> {
  // Return the data in chunks of minimum 6 chars, broken by whitespace
  let start = 0;
  let end = 0;
  while (end < data.length) {
    end = Math.min(data.length, start + 6);
    while (end < data.length && data[end] !== " " && data[end] !== "\n") {
      end++;
    }
    yield data.slice(start, end);
    start = end;
  }
}
