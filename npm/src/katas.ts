// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { katas } from "./katas-content.generated.js";

export type Example = {
  type: "example";
  id: string;
  title: string;
  contentAsHtml: string;
  contentAsMarkdown: string;
  source: string;
};

export type Exercise = {
  type: "exercise";
  id: string;
  title: string;
  contentAsHtml: string;
  contentAsMarkdown: string;
  verificationImplementation: string;
  referenceImplementation: string;
  placeholderImplementation: string;
};

export type Reading = {
  type: "reading";
  id: string;
  title: string;
  contentAsHtml: string;
  contentAsMarkdown: string;
};

export type KataItem = Example | Exercise | Reading;

export type Kata = {
  id: string;
  title: string;
  contentAsHtml: string;
  contentAsMarkdown: string;
  items: KataItem[];
};

export async function getAllKatas(): Promise<Kata[]> {
  return katas as Kata[];
}

export async function getKata(id: string): Promise<Kata> {
  const katas = await getAllKatas();
  return (
    katas.find((k) => k.id === id) ||
    Promise.reject(`Failed to get kata with id: ${id}`)
  );
}
