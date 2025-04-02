// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as qviz from "./circuit-vis";
import { useEffect, useRef, useState } from "preact/hooks";
import { CircuitProps } from "./data.js";
import { Spinner } from "./spinner.js";
import { CURRENT_VERSION } from "./circuit-vis/circuit";

// For perf reasons we set a limit on how many gates/qubits
// we attempt to render. This is still a lot higher than a human would
// reasonably want to look at, but it takes about a second to
// render a circuit this big on a mid-grade laptop so we allow it.
const MAX_OPERATIONS = 10000;
const MAX_QUBITS = 1000;

// For now we only support one circuit at a time.
const MAX_CIRCUITS = 1;

function isCircuitGroup(circuit: any): circuit is qviz.CircuitGroup {
  return circuit && Array.isArray(circuit.circuits);
}

function isCircuit(circuit: any): circuit is qviz.Circuit {
  return (
    circuit &&
    Array.isArray(circuit.qubits) &&
    Array.isArray(circuit.componentGrid)
  );
}

// This component is shared by the Python widget and the VS Code panel
export function Circuit(props: {
  circuit?: qviz.CircuitGroup | qviz.Circuit;
  isEditable: boolean;
  editCallback?: (fileData: qviz.CircuitGroup) => void;
}) {
  const emptyCircuit: qviz.Circuit = {
    qubits: [],
    componentGrid: [],
  };

  const emptyCircuitGroup: qviz.CircuitGroup = {
    version: CURRENT_VERSION,
    circuits: [emptyCircuit],
  };

  let circuitGroup: qviz.CircuitGroup;
  let circuit: qviz.Circuit;
  if (isCircuitGroup(props.circuit)) {
    circuitGroup =
      props.circuit.circuits.length === 0 ? emptyCircuitGroup : props.circuit;
    circuit = circuitGroup.circuits[0];
  } else if (isCircuit(props.circuit)) {
    circuitGroup = emptyCircuitGroup;
    circuit = props.circuit;
    circuitGroup.circuits[0] = circuit;
  } else {
    circuitGroup = emptyCircuitGroup;
    circuit = circuitGroup.circuits[0];
  }

  if (circuit.componentGrid === undefined) circuit.componentGrid = [];
  if (circuit.qubits === undefined) circuit.qubits = [];

  if (circuit.componentGrid === undefined) circuit.componentGrid = [];
  if (circuit.qubits === undefined) circuit.qubits = [];

  const unrenderable =
    circuitGroup.circuits.length > MAX_CIRCUITS ||
    (!props.isEditable && circuit.qubits.length === 0) ||
    circuit.componentGrid.length > MAX_OPERATIONS ||
    circuit.qubits.length > MAX_QUBITS;

  return (
    <div>
      {unrenderable ? (
        <Unrenderable
          qubits={circuit.qubits.length}
          operations={circuit.componentGrid.length}
        />
      ) : (
        <ZoomableCircuit {...props} circuitGroup={circuitGroup} />
      )}
    </div>
  );
}

function ZoomableCircuit(props: {
  circuitGroup: qviz.CircuitGroup;
  isEditable: boolean;
  editCallback?: (fileData: qviz.CircuitGroup) => void;
}) {
  const circuitDiv = useRef<HTMLDivElement>(null);
  const [zoomLevel, setZoomLevel] = useState(100);
  const [rendering, setRendering] = useState(true);
  const [zoomOnResize, setZoomOnResize] = useState(true);

  useEffect(() => {
    // Enable "rendering" text while the circuit is being drawn
    setRendering(true);
    const container = circuitDiv.current!;
    container.innerHTML = "";
  }, [props.circuitGroup]);

  useEffect(() => {
    if (rendering) {
      const container = circuitDiv.current!;
      // Draw the circuits - may take a while for large circuits
      const svg = renderCircuits(
        props.circuitGroup,
        container,
        props.isEditable,
        props.editCallback,
      );

      if (!props.isEditable) {
        const initialZoom = calculateZoomToFit(container, svg as SVGElement);
        // Set the initial zoom level
        setZoomLevel(initialZoom);
        // Resize the SVG to fit
        updateWidth();
      }

      // Calculate the initial zoom level based on the container width
      // Disable "rendering" text
      setRendering(false);
    } else if (!props.isEditable) {
      // Initial drawing done, attach window resize handler
      window.addEventListener("resize", onResize);
      return () => {
        window.removeEventListener("resize", onResize);
      };
    }
  }, [rendering, zoomOnResize]);

  useEffect(() => {
    updateWidth();
  }, [zoomLevel]);

  return (
    <div>
      <div>
        {props.isEditable || rendering ? null : (
          <ZoomControl zoom={zoomLevel} onInput={userSetZoomLevel} />
        )}
      </div>
      <div>
        {rendering
          ? `Rendering diagram with ${props.circuitGroup.circuits[0].componentGrid.length} gates...`
          : ""}
      </div>
      <div class="qs-circuit" ref={circuitDiv}></div>
    </div>
  );

  /**
   * Window resize handler to recalculate and set the zoom level
   * based on the new window width.
   */
  function onResize() {
    if (!zoomOnResize) {
      return;
    }

    const [container, svg] = [circuitDiv.current, currentSvg()];
    if (container && svg) {
      // Recalculate the zoom level based on the container width
      const initialZoom = calculateZoomToFit(container, svg);
      // Set the zoom level
      setZoomLevel(initialZoom);
    }
  }

  /**
   * Update the width of the SVG element based on the current zoom level.
   */
  function updateWidth() {
    const svg = currentSvg();
    if (svg) {
      // The width attribute contains the true width, generated by qviz.
      // We'll leave this attribute untouched, so we can use it again if the
      // zoom level is ever updated.
      const width = svg.getAttribute("width")!;

      // We'll set the width in the style attribute to (true width * zoom level).
      // This value takes precedence over the true width in the width attribute.
      svg.setAttribute(
        "style",
        `max-width: ${width}; width: ${(parseInt(width) * (zoomLevel || 100)) / 100}; height: auto`,
      );
    }
  }

  function renderCircuits(
    circuitGroup: qviz.CircuitGroup,
    container: HTMLDivElement,
    isEditable: boolean,
    editCallback?: (fileData: qviz.CircuitGroup) => void,
  ) {
    if (isEditable) {
      let circuitPanel = qviz.create(circuitGroup).useDraggable().usePanel();
      if (editCallback) {
        circuitPanel = circuitPanel.useOnCircuitChange(editCallback);
      }
      circuitPanel.useEvents().draw(container);
    } else {
      qviz.create(circuitGroup).draw(container);
    }
    // circuit-vis hardcodes the styles in the SVG.
    // Remove the style elements -- we'll define the styles in our own CSS.
    const styleElements = container.querySelectorAll("style");
    styleElements?.forEach((tag) => tag.remove());

    return container.getElementsByClassName("qviz")[0]!;
  }

  /**
   * Calculate the zoom level that will fit the circuit into the current size of the container.
   */
  function calculateZoomToFit(container: HTMLDivElement, svg: SVGElement) {
    const containerWidth = container.clientWidth;
    // width and height are the true dimensions generated by qviz
    const width = parseInt(svg.getAttribute("width")!);
    const height = svg.getAttribute("height")!;

    svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
    const zoom = Math.min(Math.ceil((containerWidth / width) * 100), 100);
    return zoom;
  }

  function currentSvg(): SVGElement | undefined {
    return circuitDiv.current?.querySelector(".qviz") ?? undefined;
  }

  function userSetZoomLevel(zoom: number) {
    setZoomOnResize(false);
    setZoomLevel(zoom);
  }
}

function Unrenderable(props: { qubits: number; operations: number }) {
  const errorDiv =
    props.qubits === 0 ? (
      <div>
        <p>No circuit to display. No qubits have been allocated.</p>
      </div>
    ) : props.operations > MAX_OPERATIONS ? (
      // Don't show the real number of operations here, as that number is
      // *already* truncated by the underlying circuit builder.
      <div>
        <p>
          This circuit has too many gates to display. The maximum supported
          number of gates is {MAX_OPERATIONS}.
        </p>
      </div>
    ) : props.qubits > MAX_QUBITS ? (
      <div>
        <p>
          This circuit has too many qubits to display. It has {props.qubits}{" "}
          qubits, but the maximum supported is {MAX_QUBITS}.
        </p>
      </div>
    ) : undefined;

  return <div class="qs-circuit-error">{errorDiv}</div>;
}

function ZoomControl(props: { zoom: number; onInput: (zoom: number) => void }) {
  return (
    <p>
      <label htmlFor="qs-circuit-zoom">Zoom </label>
      <input
        id="qs-circuit-zoom"
        type="number"
        min="10"
        max="100"
        step="10"
        value={props.zoom}
        onInput={(e) =>
          props.onInput(parseInt((e.target as HTMLInputElement).value) || 0)
        }
      />
      %
    </p>
  );
}

// This component is exclusive to the VS Code panel
export function CircuitPanel(props: CircuitProps) {
  const error = props.errorHtml ? (
    <div>
      <p>
        {props.circuit
          ? "The program encountered a failure. See the error(s) below."
          : "A circuit could not be generated for this program. See the error(s) below."}
        <br />
      </p>
      <div dangerouslySetInnerHTML={{ __html: props.errorHtml }}></div>
    </div>
  ) : null;

  return (
    <div class="qs-circuit-panel">
      <div>
        <h1>
          {props.title} {props.simulated ? "(Trace)" : ""}
        </h1>
      </div>
      <div class="qs-circuit-error">{error}</div>
      <p>{props.targetProfile}</p>
      <p>
        {
          props.simulated
            ? "WARNING: This diagram shows the result of tracing a dynamic circuit, and may change from run to run."
            : "\xa0" // nbsp to keep line height consistent
        }
      </p>
      <p>
        Learn more at{" "}
        <a href="https://aka.ms/qdk.circuits">https://aka.ms/qdk.circuits</a>
      </p>
      {props.calculating ? (
        <div>
          <Spinner />
        </div>
      ) : null}
      {props.circuit ? (
        <Circuit
          circuit={props.circuit}
          isEditable={props.isEditable}
          editCallback={props.editCallback}
        ></Circuit>
      ) : null}
    </div>
  );
}
