// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function toArray<T>(arrayLike: any): T[] {
  const arr: T[] = [];
  for (let i = 0, l = arrayLike.length; i < l; i++) {
    arr.push(arrayLike[i]);
  }

  return arr;
}

function px<T extends HTMLElement>(node: T, styleProperty: string) {
  const win = node.ownerDocument.defaultView || window;
  const val = win.getComputedStyle(node).getPropertyValue(styleProperty);
  return val ? parseFloat(val.replace("px", "")) : 0;
}

export function getImageSize<T extends HTMLElement>(node: T) {
  const leftBorder = px(node, "border-left-width");
  const rightBorder = px(node, "border-right-width");
  const topBorder = px(node, "border-top-width");
  const bottomBorder = px(node, "border-bottom-width");

  return {
    width: node.clientWidth + leftBorder + rightBorder,
    height: node.clientHeight + topBorder + bottomBorder + 12, // Fixes up truncated region
  };
}

export function createImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.decode = () => resolve(img) as any;
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.crossOrigin = "anonymous";
    img.decoding = "async";
    img.src = url;
  });
}

export async function svgToDataURI(svg: Element): Promise<string> {
  return Promise.resolve()
    .then(() => new XMLSerializer().serializeToString(svg))
    .then(encodeURIComponent)
    .then((html) => `data:image/svg+xml;charset=utf-8,${html}`);
}

export async function nodeToDataURI(
  node: HTMLElement,
  width: number,
  height: number,
): Promise<string> {
  const xmlns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(xmlns, "svg");
  const foreignObject = document.createElementNS(xmlns, "foreignObject");

  svg.setAttribute("width", `${width}`);
  svg.setAttribute("height", `${height}`);
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);

  foreignObject.setAttribute("width", "100%");
  foreignObject.setAttribute("height", "100%");
  foreignObject.setAttribute("x", "0");
  foreignObject.setAttribute("y", "0");
  foreignObject.setAttribute("externalResourcesRequired", "true");

  svg.appendChild(foreignObject);
  foreignObject.appendChild(node);
  return svgToDataURI(svg);
}
