// Except an array of results (strings) and generate a histogram

export type HistogramData = Array<{
    label: string;
    count: number;
}>;

// Takes an array of results (string labels) and buckets them for a histogram
export function generateHistogramData(input: string[]): HistogramData {
    // Create an array of objects with label and frequency
    let processedData: {[label: string]: number} = {};
    
    input.forEach(x => {
        if (x in processedData) {
            processedData[x] += 1;
        } else {
            processedData[x] = 1;
        }
    });

    let arrData: HistogramData = [];
    
    for(const elem in processedData) {
        arrData.push({label: elem, count: processedData[elem]});
    }
    arrData.sort( (a, b) => (a.label > b.label ? 1 : -1));
    return arrData;
}

export function generateHistogramSvg(data: HistogramData) : SVGSVGElement {
    const xmlns = "http://www.w3.org/2000/svg";
    let svgElem = document.createElementNS(xmlns, "svg") as SVGSVGElement;
    svgElem.classList.add("histogram");
    svgElem.setAttributeNS(null, "viewBox", "0 0 180 100");

    if (!data.length) return svgElem;

    // Add the initial child elements
    const g = document.createElementNS(xmlns, "g");
    g.setAttributeNS(null, "transform", "translate(10, 5)");
    svgElem.appendChild(g);

    const histoLabel = document.createElementNS(xmlns, "text");
    histoLabel.setAttributeNS(null, "id", "histo-label");
    histoLabel.setAttributeNS(null, "x", "10");
    histoLabel.setAttributeNS(null, "y", "98");
    svgElem.appendChild(histoLabel);

    const hoverText = document.createElementNS(xmlns, "text");
    hoverText.setAttributeNS(null, "id", "hover-text");
    hoverText.setAttributeNS(null, "x", "90");
    hoverText.setAttributeNS(null, "y", "10");
    svgElem.appendChild(hoverText);

    let totalCount = 0;
    let maxCount = 0;
    data.forEach(x => {
        totalCount += x.count;
        maxCount = Math.max(x.count, maxCount);
    });

    // Add a bar for each entry. Total width should be 0 to 160, with 4 space. Bar height max 72.
    let barOffset = 160 / data.length;
    let barWidth = barOffset * 0.8;
    data.forEach( (entry, idx) => {
        let height = 72 * (entry.count / maxCount);
        let x = barOffset * idx;
        let y = 87 - height;

        // Add the bar rect
        let rect = document.createElementNS(xmlns, "rect"); 
        rect.setAttributeNS(null, "class", "bar");
        rect.setAttributeNS(null, "x", `${x}`);
        rect.setAttributeNS(null, "y", `${y}`);
        rect.setAttributeNS(null, "width", `${barWidth}`);
        rect.setAttributeNS(null, "height", `${height}`);
        g.appendChild(rect);

        // Title for the rect
        let title = document.createElementNS(xmlns, "title");
        title.textContent = `${entry.label} at ${(entry.count / totalCount * 100).toFixed(2)}%`;
        rect.appendChild(title);

        // Add the text label
        let text = document.createElementNS(xmlns, "text");
        text.setAttributeNS(null, "class", "bar-label");
        text.setAttributeNS(null, "x", `${x + barWidth / 2}`);
        text.setAttributeNS(null, "y", `${y - 3}`);
        text.textContent = entry.label;
        // text.setAttributeNS(null, "content", entry.label);
        g.appendChild(text);
    });

    let currentSelected: SVGRectElement | null = null
    g.addEventListener('click', ev => {
        if (ev.target instanceof SVGRectElement) {
            let targetElem = ev.target;
            if (targetElem.classList.contains('bar')) {

                if (currentSelected) currentSelected.classList.remove('bar-selected');
                if (targetElem == currentSelected) {
                    currentSelected = null;
                    histoLabel.textContent = "";
                    // TODO: Fire filter cleared event
                    return;
                }

                targetElem.classList.add('bar-selected');
                currentSelected = targetElem;
                histoLabel.textContent =
                    "Filter: " + targetElem.querySelector('title')!.textContent;
                // TODO: Fire filter set event
            }
        }
    });

    g.addEventListener('mouseover', ev => {
        if (ev.target instanceof SVGRectElement) {
            let title = ev.target.querySelector('title')!.textContent;
            hoverText.textContent = title;
        }
    })
    g.addEventListener('mouseout', ev => {
        if (ev.target instanceof SVGRectElement) {
            hoverText.textContent = "";
        }
    })

    return svgElem;
}

export const sampleData: string[] = [
    "|000⟩","|000⟩", 
    "|001⟩","|001⟩","|001⟩","|001⟩","|001⟩","|001⟩","|001⟩",
    // "|001⟩","|001⟩",
    "|010⟩","|010⟩",
    "|011⟩","|011⟩",
    // "|011⟩","|011⟩","|011⟩","|011⟩","|011⟩","|011⟩","|011⟩","|011⟩",
    "|100⟩","|100⟩",
    "|101⟩",
    "|110⟩","|110⟩",
    "|111⟩","|111⟩",
];
