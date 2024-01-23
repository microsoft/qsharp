// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { CreateIntegerTicks, CreateTimeTicks, Tick } from "../src/ux/ticks.js";

export type ScatterSeries = {
  color: string;
  items: PlotItem[];
};

export type PlotItem = {
  x: number;
  y: number;
  label: string;
};

export type Axis = {
  isTime: boolean;
  label: string;
};

type Range = {
  min: number;
  max: number;
};

export function HideTooltip() {
  const tooltip = document.getElementById("tooltip");
  if (tooltip) {
    tooltip.setAttribute("visibility", "hidden");
  }
}

function drawTooltip(target: SVGCircleElement, clicked: boolean = false) {
  const xAttr = target.getAttribute("cx");
  const x = xAttr ? parseInt(xAttr) : 0;
  const yAttr = target.getAttribute("cy");
  const y = yAttr ? parseInt(yAttr) : -0;
  const text = target.getAttribute("data-label");
  const tooltipTextLeftPadding = 5;
  const tooltipRectanglePaddingHeight = 10;
  const tooltipTextPaddingHeight = 25;
  const tooltip = document.getElementById("tooltip");
  const tooltipRect = document.getElementById("tooltipRect");
  const tooltipText = document.getElementById(
    "tooltipText",
  ) as unknown as SVGTextElement;

  if (tooltipText) {
    tooltipText.setAttribute("x", (x + tooltipTextLeftPadding).toString());
    tooltipText.setAttribute("y", (y + tooltipTextPaddingHeight).toString());
    tooltipText.textContent = text;
  }
  if (tooltipRect && tooltipText) {
    const box = tooltipText.getBBox();
    const textWidth = box.width;
    tooltipRect.setAttribute(
      "width",
      (textWidth + 2 * tooltipTextLeftPadding).toString(),
    );
    tooltipRect.setAttribute("x", x.toString());
    tooltipRect.setAttribute(
      "y",
      (y + tooltipRectanglePaddingHeight).toString(),
    );
  }
  if (tooltip) {
    tooltip.setAttribute("visibility", "visible");
    tooltip.setAttribute("clicked", clicked.toString());
  }
}

function hideTooltipIfNotClicked() {
  const tooltip = document.getElementById("tooltip");
  if (tooltip) {
    if (tooltip.getAttribute("clicked") === "false") {
      tooltip.setAttribute("visibility", "hidden");
    }
  }
}

function deselectPoint() {
  const chart = document.getElementById(`scatterChart`);
  if (chart) {
    if (chart.getAttribute("selectedPoint")) {
      const point = document.getElementById(
        chart.getAttribute("selectedPoint") as string,
      );
      if (point) {
        point.classList.remove("qs-scatterChart-point-selected");
      }
    }
  }
}

export function SelectPoint(seriesIndex: number, pointIndex: number) {
  deselectPoint();
  const point = document.getElementById(`point-${seriesIndex}-${pointIndex}`);
  const chart = document.getElementById(`scatterChart`);
  if (point && chart) {
    point.classList.add("qs-scatterChart-point-selected");
    chart.setAttribute("selectedPoint", point.id);
    drawTooltip(point as unknown as SVGCircleElement, true);
  }
}

export function ScatterChart(props: {
  data: ScatterSeries[];
  xAxis: Axis;
  yAxis: Axis;
  onPointSelected(seriesIndex: number, pointIndex: number): void;
}) {
  const data = props.data;

  function findMinMaxSingle(
    series: ScatterSeries,
  ): [number, number, number, number] {
    const xs = series.items.map((item) => item.x);
    const ys = series.items.map((item) => item.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    return [minX, maxX, minY, maxY];
  }

  function findMinMaxAll(
    series: ScatterSeries[],
  ): [number, number, number, number] {
    const minMax = series.map(findMinMaxSingle);
    const minX = Math.min(...minMax.map((x) => x[0]));
    const maxX = Math.max(...minMax.map((x) => x[1]));
    const minY = Math.min(...minMax.map((x) => x[2]));
    const maxY = Math.max(...minMax.map((x) => x[3]));
    return [minX, maxX, minY, maxY];
  }

  const [minX, maxX, minY, maxY] = findMinMaxAll(data);

  const rangeCoefficient = 2;
  const rangeX: Range = {
    min: minX / rangeCoefficient,
    max: maxX * rangeCoefficient,
  };
  const rangeY: Range = {
    min: minY / rangeCoefficient,
    max: maxY * rangeCoefficient,
  };

  function createAxisTicks(range: Range, isTime: boolean): Tick[] {
    if (isTime) {
      return CreateTimeTicks(range.min, range.max);
    } else {
      return CreateIntegerTicks(range.min, range.max);
    }
  }

  const xTicks = createAxisTicks(rangeX, props.xAxis.isTime);
  const yTicks = createAxisTicks(rangeY, props.yAxis.isTime);

  function coordinateToSvgLogarithmic(
    value: number,
    range: Range,
    size: number,
  ): number {
    return (
      ((Math.log(value) - Math.log(range.min)) /
        (Math.log(range.max) - Math.log(range.min))) *
      size
    );
  }

  const yAxisTitleWidth = 20;
  const yAxisTickCaptionMaxWidth = 100;
  const axisTickLength = 5;
  const axisLineWidth = 1;
  const xLeftMargin =
    yAxisTitleWidth + yAxisTickCaptionMaxWidth + axisTickLength + axisLineWidth;
  const xRightMargin = 160; // to show tooltips on the right hand side. If we can move tooltips dynamically, we can get rid of this.

  const axisTitleHeight = 20;
  const xAxisTickCaptionMaxHeight = 16;
  const yMargin =
    axisTitleHeight +
    xAxisTickCaptionMaxHeight +
    axisTickLength +
    axisLineWidth;

  const svgWidth = 960;
  const svgViewBoxWidthPadding = 50;
  const svgHeight = 480;
  const svgViewBoxHeightPadding = 10;
  const svgXMin = -xLeftMargin;

  const plotAreaWidth = svgWidth - xLeftMargin - xRightMargin;
  const plotAreaHeight = svgHeight - yMargin;

  const viewBox = `${svgXMin - svgViewBoxWidthPadding} ${
    -plotAreaHeight - svgViewBoxHeightPadding
  } ${svgWidth + svgViewBoxWidthPadding} ${
    svgHeight + svgViewBoxHeightPadding
  }`;

  const yAxisTextPaddingFromTicks = 5;
  const yAxisTextYPadding = 6;

  return (
    <div style="display: flex; flex-wrap: wrap; margin-top: 8px;">
      <div class="chart-container">
        <svg
          width={svgWidth}
          height={svgHeight}
          viewBox={viewBox}
          id="scatterChart"
        >
          <line
            id="xAxis"
            x1="0"
            y1="0"
            x2={plotAreaWidth}
            y2="0"
            stroke="var(--border-color)"
          />

          {xTicks.map((tick) => {
            const x = coordinateToSvgLogarithmic(
              tick.value,
              rangeX,
              plotAreaWidth,
            );
            return (
              <g>
                <line
                  x1={x}
                  y1="1"
                  x2={x}
                  y2={axisTickLength}
                  stroke="var(--border-color)"
                />
                <text
                  x={x}
                  y={axisTickLength + xAxisTickCaptionMaxHeight}
                  text-anchor="middle"
                  fill="var(--main-color)"
                >
                  {tick.label}
                </text>
              </g>
            );
          })}

          <line
            id="yAxis"
            x1="0"
            y1="0"
            x2="0"
            y2={-svgHeight}
            stroke="var(--border-color)"
          />

          {yTicks.map((tick) => {
            const y = -coordinateToSvgLogarithmic(
              tick.value,
              rangeY,
              plotAreaHeight,
            );
            return (
              <g>
                <line
                  x1="0"
                  y1={y}
                  x2={-axisTickLength}
                  y2={y}
                  stroke="var(--border-color)"
                />
                <text
                  x={-axisTickLength - yAxisTextPaddingFromTicks}
                  y={y + yAxisTextYPadding}
                  text-anchor="end"
                  fill="var(--main-color)"
                >
                  {tick.label}
                </text>
              </g>
            );
          })}

          <text
            x={plotAreaWidth / 2}
            y={yMargin}
            class="qs-scatterChart-x-axisTitle"
          >
            {props.xAxis.label} (logarithmic)
          </text>

          <text
            x={xLeftMargin - axisTitleHeight}
            y={plotAreaHeight / 2}
            class="qs-scatterChart-y-axisTitle"
          >
            {props.yAxis.label} (logarithmic)
          </text>

          <text
            class="qs-scatterChart-watermark"
            x={xLeftMargin - axisTitleHeight}
            y={-svgHeight + yMargin}
          >
            Created with Azure Quantum Resource Estimator
          </text>

          {data.map((data, seriesIndex) => {
            return data.items.map((item, pointIndex) => {
              const x = coordinateToSvgLogarithmic(
                item.x,
                rangeX,
                plotAreaWidth,
              );

              const y = -coordinateToSvgLogarithmic(
                item.y,
                rangeY,
                plotAreaHeight,
              );
              return (
                <circle
                  id={`point-${seriesIndex}-${pointIndex}`}
                  cx={x}
                  cy={y}
                  data-label={item.label}
                  class="qs-scatterChart-point"
                  stroke={data.color}
                  onMouseOver={(e) => {
                    drawTooltip(e.currentTarget, false);
                    deselectPoint();
                  }}
                  onClick={() => {
                    SelectPoint(seriesIndex, pointIndex);
                    props.onPointSelected(seriesIndex, pointIndex);
                  }}
                  onMouseOut={() => hideTooltipIfNotClicked()}
                />
              );
            });
          })}
          <g id="tooltip" visibility="hidden">
            <rect
              id="tooltipRect"
              x="100"
              y="-100"
              width="200"
              height="22"
              fill="white"
              stroke="black"
              stroke-width="1"
            />
            <text
              id="tooltipText"
              x="105"
              y="115"
              text-anchor="left"
              fill="black"
            ></text>
          </g>
        </svg>
      </div>
    </div>
  );
}
