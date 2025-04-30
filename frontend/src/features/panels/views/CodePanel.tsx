import { getOwner, runWithOwner, untrack } from "solid-js";

import { CodeEditor } from "@features/editor/views";

export function CodePanel(id: string) {
  const owner = getOwner();

  if (id.startsWith("file:")) {
    id = id.slice("file:".length);
  }

  // A little hack to get DOM Node from component, in DEV mode
  // `child` will be a function
  const child =  runWithOwner(owner, () => <CodeEditor file={id} />);
  return typeof child === "function" ? untrack(child) : child;
}
