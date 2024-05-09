// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { formatInputs } from './formatters/inputFormatter';
import { formatGates } from './formatters/gateFormatter';
import { formatRegisters } from './formatters/registerFormatter';
import { processOperations } from './process';
import { ConditionalRender, Circuit, Operation } from './circuit';
import { Metadata, GateType } from './metadata';
import { StyleConfig, style, STYLES } from './styles';
import { createUUID } from './utils';
import { svgNS } from './constants';

/**
 * Contains metadata for visualization.
 */
interface ComposedSqore {
    /** Width of visualization. */
    width: number;
    /** Height of visualization. */
    height: number;
    /** SVG elements the make up the visualization. */
    elements: SVGElement[];
}

/**
 * Defines the mapping of unique ID to each operation. Used for enabling
 * interactivity.
 */
type GateRegistry = {
    [id: string]: Operation;
};

/**
 * Entrypoint class for rendering circuit visualizations.
 */
export class Sqore {
    circuit: Circuit;
    style: StyleConfig = {};
    gateRegistry: GateRegistry = {};

    /**
     * Initializes Sqore object with custom styles.
     *
     * @param circuit Circuit to be visualized.
     * @param style Custom visualization style.
     */
    constructor(circuit: Circuit, style: StyleConfig | string = {}) {
        this.circuit = circuit;
        this.style = this.getStyle(style);
    }

    /**
     * Render circuit into `container` at the specified layer depth.
     *
     * @param container HTML element for rendering visualization into.
     * @param renderDepth Initial layer depth at which to render gates.
     */
    draw(container: HTMLElement, renderDepth = 0): void {
        // Inject into container
        if (container == null) throw new Error(`Container not provided.`);

        // Create copy of circuit to prevent mutation
        const circuit: Circuit = JSON.parse(JSON.stringify(this.circuit));

        // Assign unique IDs to each operation
        circuit.operations.forEach((op, i) => this.fillGateRegistry(op, i.toString()));

        // Render operations at starting at given depth
        circuit.operations = this.selectOpsAtDepth(circuit.operations, renderDepth);

        // If only one top-level operation, expand automatically:
        if (
            circuit.operations.length == 1 &&
            circuit.operations[0].dataAttributes != null &&
            circuit.operations[0].dataAttributes.hasOwnProperty('id')
        ) {
            const id: string = circuit.operations[0].dataAttributes['id'];
            this.expandOperation(circuit.operations, id);
        }

        this.renderCircuit(container, circuit);
    }

    /**
     * Retrieve style for visualization.
     *
     * @param style Custom style or style name.
     *
     * @returns Custom style.
     */
    private getStyle(style: StyleConfig | string = {}): StyleConfig {
        if (typeof style === 'string' || style instanceof String) {
            const styleName: string = style as string;
            if (!STYLES.hasOwnProperty(styleName)) {
                console.error(`No style ${styleName} found in STYLES.`);
                return {};
            }
            style = STYLES[styleName];
        }
        return style;
    }

    /**
     * Render circuit into `container`.
     *
     * @param container HTML element for rendering visualization into.
     * @param circuit Circuit object to be rendered.
     */
    private renderCircuit(container: HTMLElement, circuit: Circuit): void {
        // Create visualization components
        const composedSqore: ComposedSqore = this.compose(circuit);
        const svg: SVGElement = this.generateSvg(composedSqore);
        container.innerHTML = '';
        container.appendChild(svg);
        this.addGateClickHandlers(container, circuit);
    }

    /**
     * Generates the components required for visualization.
     *
     * @param circuit Circuit to be visualized.
     *
     * @returns `ComposedSqore` object containing metadata for visualization.
     */
    private compose(circuit: Circuit): ComposedSqore {
        const add = (acc: Metadata[], gate: Metadata | Metadata[]): void => {
            if (Array.isArray(gate)) {
                gate.forEach((g) => add(acc, g));
            } else {
                acc.push(gate);
                gate.children?.forEach((g) => add(acc, g));
            }
        };

        const flatten = (gates: Metadata[]): Metadata[] => {
            const result: Metadata[] = [];
            add(result, gates);
            return result;
        };

        const { qubits, operations } = circuit;
        const { qubitWires, registers, svgHeight } = formatInputs(qubits);
        const { metadataList, svgWidth } = processOperations(operations, registers);
        const formattedGates: SVGElement = formatGates(metadataList);
        const measureGates: Metadata[] = flatten(metadataList).filter(({ type }) => type === GateType.Measure);
        const formattedRegs: SVGElement = formatRegisters(registers, measureGates, svgWidth);

        const composedSqore: ComposedSqore = {
            width: svgWidth,
            height: svgHeight,
            elements: [qubitWires, formattedRegs, formattedGates],
        };
        return composedSqore;
    }

    /**
     * Generates visualization of `composedSqore` as an SVG.
     *
     * @param composedSqore ComposedSqore to be visualized.
     *
     * @returns SVG representation of circuit visualization.
     */
    private generateSvg(composedSqore: ComposedSqore): SVGElement {
        const { width, height, elements } = composedSqore;
        const uuid: string = createUUID();

        const svg: SVGElement = document.createElementNS(svgNS, 'svg');
        svg.setAttribute('id', uuid);
        svg.setAttribute('class', 'qviz');
        svg.setAttribute('width', width.toString());
        svg.setAttribute('height', height.toString());
        svg.style.setProperty('max-width', 'fit-content');

        // Add styles
        const css = document.createElement('style');
        css.innerHTML = style(this.style);
        svg.appendChild(css);

        // Add body elements
        elements.forEach((element: SVGElement) => svg.appendChild(element));

        return svg;
    }

    /**
     * Depth-first traversal to assign unique ID to `operation`.
     * The operation is assigned the id `id` and its `i`th child is recursively given
     * the id `${id}-${i}`.
     *
     * @param operation Operation to be assigned.
     * @param id: ID to assign to `operation`.
     *
     */
    private fillGateRegistry(operation: Operation, id: string): void {
        if (operation.dataAttributes == null) operation.dataAttributes = {};
        operation.dataAttributes['id'] = id;
        // By default, operations cannot be zoomed-out
        operation.dataAttributes['zoom-out'] = 'false';
        this.gateRegistry[id] = operation;
        operation.children?.forEach((childOp, i) => {
            this.fillGateRegistry(childOp, `${id}-${i}`);
            if (childOp.dataAttributes == null) childOp.dataAttributes = {};
            // Children operations can be zoomed out
            childOp.dataAttributes['zoom-out'] = 'true';
        });
        // Composite operations can be zoomed in
        operation.dataAttributes['zoom-in'] = (operation.children != null).toString();
    }

    /**
     * Pick out operations that are at or below `renderDepth`.
     *
     * @param operations List of circuit operations.
     * @param renderDepth Initial layer depth at which to render gates.
     *
     * @returns List of operations at or below specifed depth.
     */
    private selectOpsAtDepth(operations: Operation[], renderDepth: number): Operation[] {
        if (renderDepth < 0) throw new Error(`Invalid renderDepth of ${renderDepth}. Needs to be >= 0.`);
        if (renderDepth === 0) return operations;
        return operations
            .map((op) => (op.children != null ? this.selectOpsAtDepth(op.children, renderDepth - 1) : op))
            .flat();
    }

    /**
     * Add interactive click handlers to circuit HTML elements.
     *
     * @param container HTML element containing visualized circuit.
     * @param circuit Circuit to be visualized.
     *
     */
    private addGateClickHandlers(container: HTMLElement, circuit: Circuit): void {
        this.addClassicalControlHandlers(container);
        this.addZoomHandlers(container, circuit);
    }

    /**
     * Add interactive click handlers for classically-controlled operations.
     *
     * @param container HTML element containing visualized circuit.
     *
     */
    private addClassicalControlHandlers(container: HTMLElement): void {
        container.querySelectorAll('.classically-controlled-btn').forEach((btn) => {
            // Zoom in on clicked gate
            btn.addEventListener('click', (evt: Event) => {
                const textSvg = btn.querySelector('text');
                const group = btn.parentElement;
                if (textSvg == null || group == null) return;

                const currValue = textSvg.firstChild?.nodeValue;
                const zeroGates = group?.querySelector('.gates-zero');
                const oneGates = group?.querySelector('.gates-one');
                switch (currValue) {
                    case '?':
                        textSvg.childNodes[0].nodeValue = '1';
                        group.classList.remove('classically-controlled-unknown');
                        group.classList.remove('classically-controlled-zero');
                        group.classList.add('classically-controlled-one');
                        zeroGates?.classList.add('hidden');
                        oneGates?.classList.remove('hidden');
                        break;
                    case '1':
                        textSvg.childNodes[0].nodeValue = '0';
                        group.classList.remove('classically-controlled-unknown');
                        group.classList.add('classically-controlled-zero');
                        group.classList.remove('classically-controlled-one');
                        zeroGates?.classList.remove('hidden');
                        oneGates?.classList.add('hidden');
                        break;
                    case '0':
                        textSvg.childNodes[0].nodeValue = '?';
                        group.classList.add('classically-controlled-unknown');
                        group.classList.remove('classically-controlled-zero');
                        group.classList.remove('classically-controlled-one');
                        zeroGates?.classList.remove('hidden');
                        oneGates?.classList.remove('hidden');
                        break;
                }
                evt.stopPropagation();
            });
        });
    }

    /**
     * Add interactive click handlers for zoom-in/out functionality.
     *
     * @param container HTML element containing visualized circuit.
     * @param circuit Circuit to be visualized.
     *
     */
    private addZoomHandlers(container: HTMLElement, circuit: Circuit): void {
        container.querySelectorAll('.gate .gate-control').forEach((ctrl) => {
            // Zoom in on clicked gate
            ctrl.addEventListener('click', (ev: Event) => {
                const gateId: string | null | undefined = ctrl.parentElement?.getAttribute('data-id');
                if (typeof gateId == 'string') {
                    if (ctrl.classList.contains('gate-collapse')) {
                        this.collapseOperation(circuit.operations, gateId);
                    } else if (ctrl.classList.contains('gate-expand')) {
                        this.expandOperation(circuit.operations, gateId);
                    }
                    this.renderCircuit(container, circuit);

                    ev.stopPropagation();
                }
            });
        });
    }

    /**
     * Expand selected operation for zoom-in interaction.
     *
     * @param operations List of circuit operations.
     * @param id ID of operation to expand.
     *
     */
    private expandOperation(operations: Operation[], id: string): void {
        operations.forEach((op) => {
            if (op.conditionalRender === ConditionalRender.AsGroup) this.expandOperation(op.children || [], id);
            if (op.dataAttributes == null) return op;
            const opId: string = op.dataAttributes['id'];
            if (opId === id && op.children != null) {
                op.conditionalRender = ConditionalRender.AsGroup;
                op.dataAttributes['expanded'] = 'true';
            }
        });
    }

    /**
     * Collapse selected operation for zoom-out interaction.
     *
     * @param operations List of circuit operations.
     * @param id ID of operation to collapse.
     *
     */
    private collapseOperation(operations: Operation[], parentId: string): void {
        operations.forEach((op) => {
            if (op.conditionalRender === ConditionalRender.AsGroup) this.collapseOperation(op.children || [], parentId);
            if (op.dataAttributes == null) return op;
            const opId: string = op.dataAttributes['id'];
            // Collapse parent gate and its children
            if (opId.startsWith(parentId)) {
                op.conditionalRender = ConditionalRender.Always;
                delete op.dataAttributes['expanded'];
            }
        });
    }
}
