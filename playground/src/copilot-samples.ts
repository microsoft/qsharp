// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

let general = String.raw`This is a **Markdown** cell with $\frac{\pi}{4}$ LaTeX support.`;

general += `
It also includes some Q#

\`\`\`qsharp
operation HelloQ() : Unit {
    // Line comment here
    if (true) {
        Message("Hello from quantum world!");
        /* block comment here */
    } else {
        PauliX(0);
    }
}
\`\`\`

..and some Python..

\`\`\`python
import numpy as np

def hello_python():
    print("Hello from Python!")
\`\`\`
`;

export default {
  general,
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
