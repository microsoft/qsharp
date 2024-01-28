// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { utils } from "../dist/browser.js";

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

export function ScatterChart(props: {
  data: ScatterSeries[];
  xAxis: Axis;
  yAxis: Axis;
  onPointSelected(seriesIndex: number, pointIndex: number): void;
}) {
  const { rangeX, rangeY } = utils.getRanges(props.data, 2 /* coefficient */);

  function createAxisTicks(range: Range, isTime: boolean): utils.Tick[] {
    return isTime
      ? utils.CreateTimeTicks(range.min, range.max)
      : utils.CreateIntegerTicks(range.min, range.max);
  }

  const xTicks = createAxisTicks(rangeX, props.xAxis.isTime);
  const yTicks = createAxisTicks(rangeY, props.yAxis.isTime);

  function coordinateToLogarithmic(value: number, range: Range): number {
    return (
      (Math.log(value) - Math.log(range.min)) /
      (Math.log(range.max) - Math.log(range.min))
    );
  }

  function toLogX(val: number): number {
    return coordinateToLogarithmic(val, rangeX) * plotAreaWidth;
  }

  function toLogY(val: number): number {
    return -coordinateToLogarithmic(val, rangeY) * plotAreaHeight;
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
  const yAxisTextYPadding = 4;

  function onPointMouseEvent(ev: MouseEvent, eventType: string) {
    // Ensure we have a point as the target
    if (!(ev.target instanceof SVGCircleElement)) return;
    const target = ev.target as SVGCircleElement;
    if (!target.classList.contains("qs-scatterChart-point")) return;

    // Get the div enclosing the chart, and the popup child of it.
    const topDiv = target.closest("div") as HTMLDivElement;
    const popup = topDiv.querySelector(".qs-chart-popup") as HTMLDivElement;

    switch (eventType) {
      case "over":
        {
          const label = target.getAttribute("data-label");
          popup.textContent = label;
          const halfWidth = popup.offsetWidth / 2;
          const divRect = topDiv.getBoundingClientRect();
          const pointRect = target.getBoundingClientRect();
          popup.style.left = `${pointRect.left - divRect.left - halfWidth}px`;
          popup.style.top = `${pointRect.top - divRect.top + 10}px`;
          popup.style.visibility = "visible";
        }
        break;
      case "out":
        popup.style.visibility = "hidden";
        break;
      case "click":
        {
          const index = JSON.parse(target.getAttribute("data-index")!);
          props.onPointSelected(index[0], index[1]);
        }
        break;
      default:
        console.error("Unknown event type: ", eventType);
    }
  }

  // The mouse events (over, out, and click) bubble, so put the hanlders on the
  // SVG element and check the target element in the handler.
  return (
    <div style="position: relative">
      <svg
        id="scatterChart"
        viewBox={viewBox}
        width={svgWidth}
        height={svgHeight}
        onMouseOver={(ev) => onPointMouseEvent(ev, "over")}
        onMouseOut={(ev) => onPointMouseEvent(ev, "out")}
        onClick={(ev) => onPointMouseEvent(ev, "click")}
      >
        <line
          class="qs-scatterChart-axis"
          x1="0"
          y1="0"
          x2={plotAreaWidth}
          y2="0"
        />

        {xTicks.map((tick) => {
          return (
            <g>
              <line
                y1="1"
                y2={axisTickLength}
                x1={toLogX(tick.value)}
                x2={toLogX(tick.value)}
                class="qs-scatterChart-tick-line"
              />
              <text
                y={axisTickLength + xAxisTickCaptionMaxHeight}
                x={toLogX(tick.value)}
                class="qs-scatterChart-x-tick-text"
              >
                {tick.label}
              </text>
            </g>
          );
        })}

        <line
          class="qs-scatterChart-axis"
          x1="0"
          y1="0"
          x2="0"
          y2={-svgHeight}
          stroke="var(--border-color)"
        />

        {yTicks.map((tick) => {
          return (
            <g>
              <line
                x1="0"
                x2={-axisTickLength}
                y1={toLogY(tick.value)}
                y2={toLogY(tick.value)}
                class="qs-scatterChart-tick-line"
              />
              <text
                x={-axisTickLength - yAxisTextPaddingFromTicks}
                y={toLogY(tick.value) + yAxisTextYPadding}
                class="qs-scatterChart-y-tick-text"
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
        <g>
          {props.data.map((series, seriesIdx) => {
            return series.items.map((plot, plotIdx) => {
              return (
                <circle
                  data-index={JSON.stringify([seriesIdx, plotIdx])}
                  data-label={plot.label}
                  class="qs-scatterChart-point"
                  cx={toLogX(plot.x)}
                  cy={toLogY(plot.y)}
                  stroke={series.color}
                />
              );
            });
          })}
        </g>
      </svg>
      <div class="qs-chart-popup"></div>
    </div>
  );
}
