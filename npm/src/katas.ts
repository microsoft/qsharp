// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { default as katasContent } from "./katas-content.generated.js";

export type QSharpSource = {
  type: "qsharp";
  sourceId: string;
  code: string;
};

export type Example = {
  type: "example";
  id: string;
  code: string; // TODO: Should use QSharpSource.
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
  code: QSharpSource;
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
  codeDependencies: string[]; // Rename to just sources.
  verificationCode: string; // Remove.
  placeholderCode: string; // Rename to placeholder.
  solutionAsHtml: string;
  solutionAsMarkdown: string;
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

export async function getExerciseDependencies(
  exercise: Exercise
): Promise<string[]> {
  return katasContent.globalCodeSources
    .filter((source) => exercise.codeDependencies.indexOf(source.name) > -1)
    .map((source) => source.contents);
}
