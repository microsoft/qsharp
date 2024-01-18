// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { SingleEstimateResult } from "./data.js";

function getPieSegment(
  x: number,
  y: number,
  radius: number,
  startAngle: number,
  endAngle: number,
  innerRadius: number,
) {
  const largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1";
  const startX = x + radius * Math.cos((Math.PI * startAngle) / 180);
  const startY = y + radius * Math.sin((Math.PI * startAngle) / 180);
  const endX = x + radius * Math.cos((Math.PI * endAngle) / 180);
  const endY = y + radius * Math.sin((Math.PI * endAngle) / 180);
  const innerStartX = x + innerRadius * Math.cos((Math.PI * startAngle) / 180);
  const innerStartY = y + innerRadius * Math.sin((Math.PI * startAngle) / 180);
  const innerEndX = x + innerRadius * Math.cos((Math.PI * endAngle) / 180);
  const innerEndY = y + innerRadius * Math.sin((Math.PI * endAngle) / 180);
  const d =
    `M ${startX} ${startY} A ${radius} ${radius} 0 ${largeArcFlag} 1 ${endX} ${endY} ` +
    `L ${innerEndX} ${innerEndY} A ${innerRadius} ${innerRadius} 0 ${largeArcFlag} 0 ${innerStartX} ${innerStartY} Z`;
  return d;
}

export function SpaceChart(props: { estimatesData: SingleEstimateResult }) {
  const breakdown = props.estimatesData.physicalCounts.breakdown;

  // The values to be shown on the pie chart
  const physicalQubitsAlgorithm = breakdown.physicalQubitsForAlgorithm;
  const physicalQubitsTFactory = breakdown.physicalQubitsForTfactories;

  // TO CHECK: Divide by 0 concern here? Is there any (valid) algorithm that could
  // be 0 physical qubits?
  const percentQubitsAlgorithm =
    physicalQubitsAlgorithm /
    (physicalQubitsAlgorithm + physicalQubitsTFactory);
  const breakAngleRaw = 360 * percentQubitsAlgorithm;

  // The pie chart doesn't render correctly if the angle is 0 or 360
  const breakAngle =
    breakAngleRaw >= 360 ? 359.9 : breakAngleRaw <= 0 ? 0.1 : breakAngleRaw;

  const numTFactories = breakdown.numTfactories;
  const numQubitsPerTFactory = Math.round(
    physicalQubitsTFactory / numTFactories,
  );

  return (
    <div style="display: flex; flex-wrap: wrap; margin-top: 8px;">
      <svg
        class="qs-widget-spaceChart"
        width="400"
        height="400"
        viewBox="50 0 450 450"
        id="pieChart"
      >
        <path
          d={getPieSegment(250, 185, 180, 0, breakAngle, 120)}
          fill="var(--vscode-charts-orange, orange)"
          stroke="white"
        ></path>
        <path
          d={getPieSegment(250, 185, 180, breakAngle, 360, 120)}
          fill="var(--vscode-charts-blue, blue)"
          stroke="white"
        ></path>
        <text x="250" y="180" text-anchor="middle" font-size="16">
          Total physical qubits
        </text>
        <text x="250" y="220" text-anchor="middle" font-size="32">
          {props.estimatesData.physicalCountsFormatted.physicalQubits}
        </text>
        <rect
          x="125"
          y="400"
          width="25"
          height="25"
          fill="var(--vscode-charts-orange, orange)"
          stroke="white"
          stroke-width="1"
        />
        <text x="155" y="408" text-anchor="start" font-size="12">
          Algorithm qubits
        </text>
        <text x="155" y="425" text-anchor="start" font-size="16">
          {physicalQubitsAlgorithm.toLocaleString()}
        </text>
        <rect
          x="275"
          y="400"
          width="25"
          height="25"
          fill="var(--vscode-charts-blue, blue)"
          stroke="white"
          stroke-width="1"
        />
        <text x="305" y="408" text-anchor="start" font-size="12">
          T factory qubits
        </text>
        <text x="305" y="425" text-anchor="start" font-size="16">
          {physicalQubitsTFactory.toLocaleString()}
        </text>
      </svg>
      <div class="spaceReport">
        <div class="spaceReportHeader">Physical resource estimates</div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Total physical qubits</div>
          <div>
            {props.estimatesData.physicalCounts.physicalQubits.toLocaleString()}
          </div>
        </div>
        <div class="spaceReportHeader">T factory parameters</div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Physical T factory qubits</div>
          <div>{breakdown.physicalQubitsForTfactories.toLocaleString()}</div>
        </div>
        <div class="spaceReportHeader">Resource estimation breakdown</div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">T factory copies</div>
          <div>{breakdown.numTfactories.toLocaleString()}</div>
        </div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Physical qubits per T factory</div>
          <div>{numQubitsPerTFactory.toLocaleString()}</div>
        </div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Physical algorithmic qubits</div>
          <div>{physicalQubitsAlgorithm.toLocaleString()}</div>
        </div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Logical algorithmic qubits</div>
          <div>{breakdown.algorithmicLogicalQubits.toLocaleString()}</div>
        </div>
        <div class="spaceReportHeader">Logical qubit parameters</div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Physical qubits</div>
          <div>
            {props.estimatesData.logicalQubit.physicalQubits.toLocaleString()}
          </div>
        </div>
      </div>
    </div>
  );
}
