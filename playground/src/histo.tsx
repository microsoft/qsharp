// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useState } from "preact/hooks";

export function Histogram(props: {data: Map<string, number>, filter: string, onFilter: (filter:string) => void }) {
    const [hoverLabel, setHoverLabel] = useState("");

    let barArray = [...props.data.entries()].sort( (a, b) => a[0] < b[0] ? -1: 1);

    let totalCount = 0;
    let maxCount = 0;
    barArray.forEach(x => {
        totalCount += x[1];
        maxCount = Math.max(x[1], maxCount);
    });

    function onMouseOverRect(evt: MouseEvent) {
        const target = evt.target! as SVGRectElement;
        const title = target.querySelector('title')?.textContent;
        setHoverLabel(title || "");
    }

    function onMouseOutRect(evt: MouseEvent) {
        setHoverLabel("");
    }
    
    function onClickRect(evt: MouseEvent) {
        const targetElem = evt.target! as SVGRectElement;
        const labelClicked = (targetElem.nextSibling! as SVGTextElement).textContent!;

        if (labelClicked === props.filter) {
            // Clicked the already selected bar. Clear the filter
            props.onFilter("");
        } else {
            props.onFilter(labelClicked!)
        }
    }

    // Add a bar for each entry. Total width should be 0 to 160, with 4 space. Bar height max 72.
    let barOffset = 160 / barArray.length;
    let barWidth = barOffset * 0.8;

    let histogramLabel = "";

    return (
        <svg class="histogram" viewBox="0 0 165 100">
            <g transform="translate(5,4)">
            { 
                barArray.map( (entry, idx) => {
                    let height = 72 * (entry[1] / maxCount);
                    let x = barOffset * idx;
                    let y = 87 - height;
                    let barClass = "bar";
                    let barLabel = `${entry[0]} at ${(entry[1] / totalCount * 100).toFixed(2)}%`;

                    if (entry[0] === props.filter) {
                        barClass += " bar-selected";
                        histogramLabel = barLabel;
                    }

                    return (<>
                        <rect class={barClass} x={x} y={y} width={barWidth} height={height}
                                onMouseOver={onMouseOverRect} onMouseOut={onMouseOutRect} onClick={onClickRect}>
                            <title>{barLabel}</title>
                        </rect>
                        <text class="bar-label" x={x + barWidth / 2} y={y - 3}>{entry[0]}</text>
                    </>);
                })
            }
            </g>
            <text class="histo-label" x="5" y="98">{histogramLabel ? `Filter: ${histogramLabel}` : null}</text>
            <text class="hover-text" x="90" y="8">{hoverLabel}</text>
        </svg>
    );
}
