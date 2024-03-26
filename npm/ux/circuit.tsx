// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as qviz from "@microsoft/quantum-viz.js/lib";
import { useEffect, useRef } from "preact/hooks";

// For perf reasons we set a limit on how many gates/qubits
// we attempt to render. This is still a lot higher than a human would
// reasonably want to look at, but it takes about a second to
// render a circuit this big on a mid-grade laptop so we allow it.
const MAX_OPERATIONS = 10000;
const MAX_QUBITS = 1000;

/* This component is shared by the Python widget and the VS Code panel */
export function Circuit(props: { circuit: qviz.Circuit }) {
  const errorDiv =
    props.circuit.qubits.length === 0 ? (
      <div>
        <p>No circuit to display. No qubits have been allocated.</p>
        <p>
          <em>
            Tip: you can generate a circuit diagram for any operation that takes
            qubits or arrays of qubits as input.
          </em>
        </p>
      </div>
    ) : props.circuit.operations.length > MAX_OPERATIONS ? (
      <div>
        <p>
          This circuit has too many gates to display. It has{" "}
          {props.circuit.operations.length} gates, but the maximum supported is{" "}
          {MAX_OPERATIONS}.
        </p>
      </div>
    ) : props.circuit.qubits.length > MAX_QUBITS ? (
      <div>
        <p>
          This circuit has too many qubits to display. It has{" "}
          {props.circuit.qubits.length} qubits, but the maximum supported is{" "}
          {MAX_QUBITS}.
        </p>
      </div>
    ) : undefined;

  if (errorDiv) {
    return <div class=".qs-circuit-error">{errorDiv}</div>;
  }

  const circuitDiv = useRef<HTMLDivElement>(null);

  useEffect(() => {
    qviz.draw(props.circuit, circuitDiv.current!);

    // quantum-viz hardcodes the styles in the SVG.
    // Remove the style elements -- we'll define the styles in our own CSS.
    const styleElements = circuitDiv.current?.querySelectorAll("style");
    styleElements?.forEach((tag) => tag.remove());
  }, [props.circuit]);

  return <div class="qs-circuit" ref={circuitDiv}></div>;
}

/* This component is exclusive to the VS Code panel */
export function CircuitPanel(props: {
  title: string;
  subtitle: string;
  circuit?: qviz.Circuit;
  errorHtml?: string;
}) {
  return (
    <div>
      <div>
        <h1>{props.title}</h1>
      </div>
      {props.circuit ? <Circuit circuit={props.circuit}></Circuit> : null}
      <div class="qs-circuit-error">
        {props.errorHtml ? (
          <div>
            <p>
              A circuit could not be generated for this program. See the
              error(s) below.
              <br />
            </p>
            <div dangerouslySetInnerHTML={{ __html: props.errorHtml }}></div>
          </div>
        ) : null}
      </div>
      <p>{props.subtitle /* target profile */}</p>
    </div>
  );
}

export type CircuitData = qviz.Circuit;
