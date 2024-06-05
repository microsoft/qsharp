// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Writes resource estimator output to PNG file

import {
  getImageSize,
  createImage,
  nodeToDataURI,
  toArray,
} from "./saveImageUtil.js";

async function cloneSingleNode<T extends HTMLElement>(
  node: T,
): Promise<HTMLElement> {
  return node.cloneNode(false) as T;
}

async function cloneChildren<T extends HTMLElement>(
  nativeNode: T,
  clonedNode: T,
): Promise<T> {
  let children: T[] = [];
  children = toArray<T>((nativeNode.shadowRoot ?? nativeNode).childNodes);

  // Depth-first traversal of DOM objects
  await children.reduce(
    (deferred, child) =>
      deferred
        .then(() => cloneNode(child))
        .then((clonedChild: HTMLElement | null) => {
          if (clonedChild) {
            clonedNode.appendChild(clonedChild);
          }
        }),
    Promise.resolve(),
  );

  return clonedNode;
}

function cloneCSSStyle<T extends HTMLElement>(nativeNode: T, clonedNode: T) {
  const targetStyle = clonedNode.style;
  if (!targetStyle) {
    return;
  }

  const sourceStyle = window.getComputedStyle(nativeNode);
  toArray<string>(sourceStyle).forEach((name) => {
    const value = sourceStyle.getPropertyValue(name);
    targetStyle.setProperty(name, value, sourceStyle.getPropertyPriority(name));
  });
}

function decorate<T extends HTMLElement>(nativeNode: T, clonedNode: T): T {
  cloneCSSStyle(nativeNode, clonedNode);
  return clonedNode;
}

export async function cloneNode<T extends HTMLElement>(
  node: T,
): Promise<T | null> {
  return Promise.resolve(node)
    .then((clonedNode) => cloneSingleNode(clonedNode) as Promise<T>)
    .then((clonedNode) => cloneChildren(node, clonedNode))
    .then((clonedNode) => decorate(node, clonedNode));
}

export async function toSvg<T extends HTMLElement>(
  node: T,
  width: number,
  height: number,
): Promise<string> {
  const clonedNode = (await cloneNode(node)) as HTMLElement;
  const dataURI = await nodeToDataURI(clonedNode, width, height);
  return dataURI;
}

export async function saveToPng<T extends HTMLElement>(
  node: T,
  backgroundColor: string,
): Promise<string> {
  let { width, height } = getImageSize(node);
  height += 10; // Hack to include cutoff region
  const svg = await toSvg(node, width, height);
  const img = await createImage(svg);

  const ratio = window.devicePixelRatio || 1;
  const canvas = document.createElement("canvas");
  canvas.width = width * ratio;
  canvas.height = height * ratio;

  const context = canvas.getContext("2d")!;
  context.fillStyle = backgroundColor;
  context.fillRect(0, 0, canvas.width, canvas.height);

  context.drawImage(img, 0, 0, canvas.width, canvas.height);
  return canvas.toDataURL();
}
