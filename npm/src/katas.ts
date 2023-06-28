// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { katasContent } from "./katas-content.generated.js";

export type Example = {
  type: "example";
  id: string;
  code: string;
};

export type Exercise = {
  type: "exercise";
  id: string;
  codeDependencies: string[];
  verificationCode: string;
  placeholderCode: string;
  solutionCode: string;
  solutionDescriptionAsHtml: string;
  solutionDescriptionAsMarkdown: string;
};

export type Text = {
  type: "text";
  contentAsHtml: string;
  contentAsMarkdown: string;
};

export type KataSection = Example | Exercise | Text;

export type Kata = {
  id: string;
  title: string;
  sections: KataSection[];
};

export async function getAllKatas(): Promise<Kata[]> {
  return katasContent.katas as Kata[];
}

export async function getKata(id: string): Promise<Kata> {
  const katas = await getAllKatas();
  return (
    katas.find((k) => k.id === id) ||
    Promise.reject(`Failed to get kata with id: ${id}`)
  );
}

export async function getExerciseDependencies(
  exercise: Exercise
): Promise<string[]> {
  const allDependencies = katasContent.codeDependencies;
  return allDependencies
    .filter(
      (dependency) => exercise.codeDependencies.indexOf(dependency.name) > -1
    )
    .map((item) => item.contents);
}
