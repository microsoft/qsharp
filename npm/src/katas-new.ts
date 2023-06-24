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
  soulutionDescriptionAsHtml: string;
  soulutionDescriptionAsMarkdown: string;
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

export type GlobalCodeSource = {
  id: string;
  code: string;
};
