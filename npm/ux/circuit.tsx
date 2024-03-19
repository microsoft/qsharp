// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as qviz from "@microsoft/quantum-viz.js/lib";
import { useEffect, useRef } from "preact/hooks";

export function Circuit(props: { circuit: qviz.Circuit }) {
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
        <h2>{props.subtitle}</h2>
      </div>
      {props.circuit ? <Circuit circuit={props.circuit}></Circuit> : null}
      <div>
        Tip: you can generate a circuit diagram for any operation that takes
        qubits or arrays of qubits as input.
      </div>
      <div>{props.errorHtml ? <div>{props.errorHtml}</div> : null}</div>
    </div>
  );
}

export type CircuitData = qviz.Circuit;
