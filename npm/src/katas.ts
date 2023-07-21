// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { default as katasContent } from "./katas-content.generated.js";

export type Example = {
  type: "example";
  id: string;
  code: string;
};

export type Text = {
  type: "text";
  contentAsHtml: string;
  contentAsMarkdown: string;
};

export type LessonItem = Example | Text;

export type Lesson = {
  type: "reading";
  id: string;
  items: LessonItem[];
};

export type Solution = {
  type: "solution";
  id: string;
  code: string;
};

export type ExplainedSolutionItem = Example | Solution | Text;

export type ExplainedSolution = {
  type: "explained-solution";
  items: ExplainedSolutionItem[];
};

export type Exercise = {
  type: "exercise";
  id: string;
  // TODO: fields that represent Q# code should be of QSharp type.
  sourceIds: string[];
  placeholderCode: string;
  explainedSolution: ExplainedSolution;
};

// TODO: Should be Exercise | Lesson | Question
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

export async function getExerciseSources(
  exercise: Exercise
): Promise<string[]> {
  return katasContent.globalCodeSources
    .filter((source) => exercise.sourceIds.indexOf(source.id) > -1)
    .map((source) => source.code);
}
