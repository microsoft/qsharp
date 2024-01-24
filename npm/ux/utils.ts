// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function findParentByClassName(
  ev: Event,
  className: string,
): Element | undefined {
  // both HTMLElement and SVGElement inherit from Element and it supports parentElement and QuerySelectors
  let element = ev.currentTarget as Element | null;
  while (element) {
    if (element.classList.contains(className)) {
      return element;
    }
    element = element.parentElement as Element | null;
  }

  return undefined;
}
