// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

function getPieSegment(
  x: number,
  y: number,
  radius: number,
  startAngle: number,
  endAngle: number,
  innerRadius: number
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

export function SpaceChart() {
  return (
    <div style="display: flex; flex-wrap: wrap;">
      <svg width="500" height="500" id="pieChart">
        <path
          d={getPieSegment(250, 185, 180, 270, 45, 120)}
          fill="orange"
          stroke="white"
        ></path>
        <path
          d={getPieSegment(250, 185, 180, 45, 270, 120)}
          fill="blue"
          stroke="white"
        ></path>
        <text x="250" y="180" text-anchor="middle" font-size="16">
          Total physical qubits
        </text>
        <text x="250" y="220" text-anchor="middle" font-size="32">
          173,592
        </text>
        <rect
          x="125"
          y="400"
          width="25"
          height="25"
          fill="orange"
          stroke="white"
          stroke-width="1"
        />
        <text x="155" y="408" text-anchor="start" font-size="12">
          Algorithm qubits
        </text>
        <text x="155" y="425" text-anchor="start" font-size="16">
          28,392
        </text>
        <rect
          x="275"
          y="400"
          width="25"
          height="25"
          fill="blue"
          stroke="white"
          stroke-width="1"
        />
        <text x="305" y="408" text-anchor="start" font-size="12">
          T factory qubits
        </text>
        <text x="305" y="425" text-anchor="start" font-size="16">
          145,200
        </text>
      </svg>
      <div class="spaceReport">
        <div class="spaceReportHeader">Physical resource estimates</div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Total physical qubits</div>
          <div>173,592</div>
        </div>
        <div class="spaceReportHeader">T factory parameters</div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Physical T factory qubits</div>
          <div>145,200</div>
        </div>
        <div class="spaceReportHeader">Resource estimation breakdown</div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">T factory copies</div>
          <div>15</div>
        </div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Physical qubits per T factory</div>
          <div>9,680</div>
        </div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Physical algorithmic qubits</div>
          <div>28,392</div>
        </div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Logical algorithmic qubits</div>
          <div>84</div>
        </div>
        <div class="spaceReportHeader">Logical qubit parameters</div>
        <div class="spaceReportRow">
          <div class="spaceDetailText">Physical qubits</div>
          <div>338</div>
        </div>
      </div>
    </div>
  );
}
