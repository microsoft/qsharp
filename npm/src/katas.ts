// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { katasContent } from "./katas-content.generated.js";

export type ExampleN = {
  type: "example";
  id: string;
  code: string;
};

export type ExerciseN = {
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

export type KataSection = ExampleN | ExerciseN | Text;

export type KataN = {
  id: string;
  title: string;
  sections: KataSection[];
};

export async function getAllKatasN(): Promise<KataN[]> {
  return katasContent.katas as KataN[];
}

export async function getKataN(id: string): Promise<KataN> {
  const katas = await getAllKatasN();
  return (
    katas.find((k) => k.id === id) ||
    Promise.reject(`Failed to get kata with id: ${id}`)
  );
}

export async function getExerciseDependencies(
  exercise: ExerciseN
): Promise<string[]> {
  const allDependencies = katasContent.codeDependencies;
  return allDependencies
    .filter(
      (dependency) => exercise.codeDependencies.indexOf(dependency.name) > -1
    )
    .map((item) => item.contents);
}
