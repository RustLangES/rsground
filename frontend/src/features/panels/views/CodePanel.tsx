import { getOwner, runWithOwner } from "solid-js";

import { CodeEditor } from "@features/editor/views";

export function CodePanel(id: string) {
  const owner = getOwner();

  if (id.startsWith("file:")) {
    id = id.slice("file:".length);
  }

  // @ts-expect-error - obviously `<CodeEditor />` don't seems like a function
  // but as this component is just a proxy and we need the HTMLElement
  // then we need to "unwrap" the component because will be
  // generated as a signal instead of a node
  return runWithOwner(owner, <CodeEditor file={id} />);
}
