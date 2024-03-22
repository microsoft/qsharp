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
