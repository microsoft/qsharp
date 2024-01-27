// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { CreateIntegerTicks, CreateTimeTicks, Tick } from "../src/ux/ticks.js";
import { useEffect, useRef } from "preact/hooks";

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

export function HideTooltip(root: Element) {
  // could be called extenally with a root out of the chart.
  const chart =
    root.id == "scatterChart" ? root : root.querySelector("#scatterChart");

  chart
    ?.querySelector("#tooltip-selected")
    ?.setAttribute("visibility", "hidden");
}

function drawTooltip(
  target: SVGCircleElement,
  root: Element,
  clicked: boolean = false,
) {
  const xAttr = target.getAttribute("cx");
  const x = xAttr ? parseInt(xAttr) : 0;
  const yAttr = target.getAttribute("cy");
  const y = yAttr ? parseInt(yAttr) : -0;
  const text = target.getAttribute("data-label");
  const tooltipTextLeftPadding = 5;
  const tooltipRectanglePaddingHeight = 10;
  const tooltipTextPaddingHeight = 25;
  const tooltipId = clicked ? "#tooltip-selected" : "#tooltip-hover";
  const tooltip = root.querySelector(tooltipId);
  const tooltipRect = tooltip?.querySelector("#tooltipRect");
  const tooltipText = tooltip?.querySelector(
    "#tooltipText",
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

function deselectPoint(root: Element) {
  if (root.getAttribute("selectedPoint")) {
    const point = root.querySelector(
      ("#" + root.getAttribute("selectedPoint")) as string,
    );
    if (point) {
      point.classList.remove("qs-scatterChart-point-selected");
    }
  }
}

export function SelectPoint(
  seriesIndex: number,
  pointIndex: number,
  root: Element,
) {
  // could be called extenally with a root out of the chart.
  const chart =
    root.id == "scatterChart" ? root : root.querySelector("#scatterChart");
  if (chart == null) {
    return;
  }
  deselectPoint(chart);
  const point = chart.querySelector(`#point-${seriesIndex}-${pointIndex}`);
  if (point) {
    point.classList.add("qs-scatterChart-point-selected");
    chart.setAttribute("selectedPoint", point.id);
    drawTooltip(point as unknown as SVGCircleElement, root, true);
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
  const yAxisTextYPadding = 4;

  function trySetAttribute(element: Element, attribute: string, value: string) {
    if (element.getAttribute(attribute) !== value) {
      element.setAttribute(attribute, value);
    }
  }

  function hideHoverTooltip() {
    const tooltip = chart?.querySelector("#tooltip-hover");

    if (tooltip) {
      if (tooltip.getAttribute("clicked") === "false") {
        tooltip.setAttribute("visibility", "hidden");
      }
    }
  }

  const scatterChartContainerRef = useRef<SVGSVGElement | null>(null);
  let chart: Element | null = null;

  function updateCoordinates() {
    if (!chart) {
      return;
    }

    chart.querySelectorAll("[x-data-value]").forEach((element) => {
      const value = Number(element.getAttribute("x-data-value"));
      const x = coordinateToSvgLogarithmic(value, rangeX, plotAreaWidth);
      const padding = element.getAttribute("x-data-padding");
      const value_with_padding = (x + Number(padding)).toString();
      trySetAttribute(element, "x", value_with_padding);
      trySetAttribute(element, "x1", value_with_padding);
      trySetAttribute(element, "x2", value_with_padding);
      trySetAttribute(element, "cx", value_with_padding);
    });

    chart.querySelectorAll("[y-data-value]").forEach((element) => {
      const value = Number(element.getAttribute("y-data-value"));
      const y = -coordinateToSvgLogarithmic(value, rangeY, plotAreaHeight);
      const padding = element.getAttribute("y-data-padding");
      const value_with_padding = (y + Number(padding)).toString();
      trySetAttribute(element, "y", value_with_padding);
      trySetAttribute(element, "y1", value_with_padding);
      trySetAttribute(element, "y2", value_with_padding);
      trySetAttribute(element, "cy", value_with_padding);
    });
  }

  useEffect(() => {
    chart = scatterChartContainerRef.current as Element | null;
    updateCoordinates();
  });

  return (
    <div style="display: flex; flex-wrap: wrap; margin-top: 8px;">
      <div class="chart-container">
        <svg
          id="scatterChart"
          ref={scatterChartContainerRef}
          viewBox={viewBox}
          width={svgWidth}
          height={svgHeight}
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
                  x-data-value={tick.value}
                  class="qs-scatterChart-tick-line"
                />
                <text
                  y={axisTickLength + xAxisTickCaptionMaxHeight}
                  x-data-value={tick.value}
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
                  y-data-value={tick.value}
                  class="qs-scatterChart-tick-line"
                />
                <text
                  x={-axisTickLength - yAxisTextPaddingFromTicks}
                  y-data-value={tick.value}
                  class="qs-scatterChart-y-tick-text"
                  y-data-padding={yAxisTextYPadding}
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
            {data.map((data, seriesIndex) => {
              return data.items.map((item, pointIndex) => {
                return (
                  <circle
                    id={`point-${seriesIndex}-${pointIndex}`}
                    data-label={item.label}
                    class="qs-scatterChart-point"
                    x-data-value={item.x}
                    y-data-value={item.y}
                    stroke={data.color}
                    onMouseOver={(e) => {
                      const circle = e.currentTarget;

                      if (chart) {
                        drawTooltip(circle, chart, false);
                      }
                      circle?.parentNode?.appendChild(circle); // move the hovered cicrle up on the rendering stack
                    }}
                    onClick={() => {
                      if (chart) {
                        SelectPoint(seriesIndex, pointIndex, chart);
                      }
                      props.onPointSelected(seriesIndex, pointIndex);
                    }}
                    onMouseOut={() => hideHoverTooltip()}
                  />
                );
              });
            })}
          </g>
          <g id="tooltip-selected" visibility="hidden">
            <rect id="tooltipRect" class="qs-scatterChart-tooltip-rect" />
            <text id="tooltipText" class="qs-scatterChart-tooltip-text" />
          </g>
          <g id="tooltip-hover" visibility="hidden">
            <rect id="tooltipRect" class="qs-scatterChart-tooltip-rect" />
            <text id="tooltipText" class="qs-scatterChart-tooltip-text" />
          </g>
        </svg>
      </div>
    </div>
  );
}
