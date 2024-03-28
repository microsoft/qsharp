import { useEffect, useRef } from "preact/hooks";

export function DocsDisplay(props: {
    docName: string;
    docContent: string;
  }) {

    const docsDiv = useRef<HTMLDivElement>(null);

    useEffect(() => {
      if (!docsDiv.current) return;
      docsDiv.current.innerHTML = props.docContent;
    }, [props.docName]);

    return (
        <div ref={docsDiv}></div>
    )
}
