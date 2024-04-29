// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function DocumentationView(props: {
    renderer: (input: string) => string;
    contentToRender: string;
}) {
  const renderedContent = {
    __html: props.renderer(props.contentToRender),
  };

  return <div
    dangerouslySetInnerHTML={renderedContent}
    />;
}
